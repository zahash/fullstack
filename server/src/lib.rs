mod api;
mod middleware;

#[cfg(feature = "tracing")]
mod span;

#[cfg(feature = "smtp")]
mod smtp;

use std::net::SocketAddr;

use axum::{
    Router,
    extract::FromRef,
    middleware::from_fn,
    routing::{get, post},
};
use contextual::Context;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

#[derive(Debug)]
pub struct ServerOpts {
    pub database_url: String,
    pub port: u16,

    #[cfg(feature = "rate-limit")]
    pub rate_limiter: RateLimiterConfig,

    #[cfg(feature = "serve-dir")]
    pub serve_dir: std::path::PathBuf,

    #[cfg(feature = "smtp")]
    pub smtp: SMTPConfig,
}

#[cfg(feature = "rate-limit")]
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub limit: usize,
    pub interval: std::time::Duration,
}

#[cfg(feature = "smtp")]
#[derive(Debug)]
pub struct SMTPConfig {
    pub relay: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub senders_dir: std::path::PathBuf,
    pub templates_dir: std::path::PathBuf,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::Pool<sqlx::Sqlite>,

    #[cfg(feature = "smtp")]
    pub smtp: crate::smtp::Smtp,
}

pub fn server(
    pool: sqlx::Pool<sqlx::Sqlite>,
    #[cfg(feature = "smtp")] smtp: crate::smtp::Smtp,
    #[cfg(feature = "rate-limit")] rate_limiter: crate::middleware::RateLimiter,
) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id());

    #[cfg(feature = "client-ip")]
    let middleware = middleware.layer(from_fn(middleware::mw_client_ip));

    #[cfg(feature = "tracing")]
    let middleware = middleware
        .layer(tower_http::trace::TraceLayer::new_for_http().make_span_with(span::span))
        .layer(from_fn(middleware::latency_ms));

    let middleware = middleware.layer(from_fn(middleware::mw_handle_leaked_5xx));

    #[cfg(feature = "rate-limit")]
    let middleware = middleware.layer(axum::middleware::from_fn_with_state(
        std::sync::Arc::new(rate_limiter),
        crate::middleware::mw_rate_limiter,
    ));

    let router = Router::new()
        .route(
            api::username::check_availability::PATH,
            get(api::username::check_availability::handler),
        )
        .route(
            api::email::check_availability::PATH,
            get(api::email::check_availability::handler),
        )
        .route(api::health::PATH, get(api::health::handler))
        .route(api::sysinfo::PATH, get(api::sysinfo::handler))
        .route(api::signup::PATH, post(api::signup::handler))
        .route(api::login::PATH, post(api::login::handler))
        .route(api::logout::PATH, get(api::logout::handler))
        .route(
            api::access_token::generate::PATH,
            post(api::access_token::generate::handler),
        )
        .route(
            api::access_token::verify::PATH,
            get(api::access_token::verify::handler),
        )
        .route(api::private::PATH, get(api::private::handler));

    #[cfg(feature = "smtp")]
    let router = router
        .route(
            api::email::initiate_verification::PATH,
            post(api::email::initiate_verification::handler),
        )
        .route(
            api::email::check_verification_token::PATH,
            post(api::email::check_verification_token::handler),
        );

    #[cfg(feature = "openapi")]
    let router = router.route(api::OPEN_API_DOCS_PATH, get(axum::Json(api::openapi())));

    router
        .with_state(AppState {
            pool,

            #[cfg(feature = "smtp")]
            smtp,
        })
        .layer(middleware)
}

pub async fn serve(opts: ServerOpts) -> Result<(), ServerError> {
    #[cfg(feature = "tracing")]
    tracing::info!("{:?}", opts);

    let pool = SqlitePool::connect(&opts.database_url)
        .await
        .context(format!("connect database :: {}", opts.database_url))?;

    #[cfg(feature = "smtp")]
    let smtp = crate::smtp::Smtp {
        transport: {
            #[cfg(not(feature = "smtp--no-tls"))]
            let mut transport =
                lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(
                    &opts.smtp.relay,
                )
                .context("smtp relay")?;

            #[cfg(feature = "smtp--no-tls")]
            let mut transport = {
                #[cfg(feature = "tracing")]
                tracing::warn!(
                    "SMTP is running in insecure mode (smtp-no-tls). TLS certificate validation is disabled — only use for local testing!"
                );

                lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(
                    &opts.smtp.relay,
                )
            };

            if let (Some(username), Some(password)) = (opts.smtp.username, opts.smtp.password) {
                use lettre::transport::smtp::authentication::Credentials;
                transport = transport.credentials(Credentials::new(username, password));
            }

            if let Some(port) = opts.smtp.port {
                transport = transport.port(port);
            }

            transport.build()
        },
        senders: std::sync::Arc::new(crate::smtp::SmtpSenders::new(opts.smtp.senders_dir)),
        tera: {
            let glob = opts.smtp.templates_dir.join("*.html");
            let glob_str = glob.to_string_lossy().to_string();
            let tera = tera::Tera::new(&glob_str).context("initialize Tera")?;
            std::sync::Arc::new(tera)
        },
    };

    #[cfg(feature = "rate-limit")]
    let rate_limiter =
        crate::middleware::RateLimiter::new(opts.rate_limiter.limit, opts.rate_limiter.interval);

    let server = server(
        pool,
        #[cfg(feature = "smtp")]
        smtp,
        #[cfg(feature = "rate-limit")]
        rate_limiter,
    );

    #[cfg(feature = "serve-dir")]
    let server = server.fallback_service(tower_http::services::ServeDir::new(&opts.serve_dir));

    let app = server.into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let listener = TcpListener::bind(addr)
        .await
        .context(format!("bind :: {addr}"))?;

    #[cfg(feature = "tracing")]
    tracing::info!(
        "listening on {}",
        listener.local_addr().context("local_addr")?
    );

    axum::serve(listener, app)
        .await
        .context("axum::serve")
        .map_err(|e| e.into())
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),

    #[cfg(feature = "smtp")]
    #[error("{0}")]
    SmtpTransport(#[from] contextual::Error<lettre::transport::smtp::Error>),

    #[cfg(feature = "smtp")]
    #[error("{0}")]
    Tera(#[from] contextual::Error<tera::Error>),

    #[error("{0}")]
    Io(#[from] contextual::Error<std::io::Error>),
}

impl FromRef<AppState> for sqlx::Pool<sqlx::Sqlite> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.pool.clone()
    }
}

#[cfg(feature = "rate-limit")]
impl std::str::FromStr for RateLimiterConfig {
    type Err = ParseRateLimiterConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::time::Duration;

        let Some((first, second)) = s.trim().split_once('/') else {
            return Err(ParseRateLimiterConfigError::MissingForwardSlash);
        };
        let limit = first.parse::<usize>()?;
        let interval = match second.to_lowercase().as_str() {
            "s" | "sec" | "second" | "seconds" => Duration::from_secs(1),
            "m" | "min" | "minute" | "minutes" => Duration::from_secs(60),
            "h" | "hr" | "hour" | "hours" => Duration::from_secs(60 * 60),
            _ => return Err(ParseRateLimiterConfigError::InvalidUnit),
        };
        Ok(Self { limit, interval })
    }
}

#[cfg(feature = "rate-limit")]
#[derive(thiserror::Error, Debug)]
pub enum ParseRateLimiterConfigError {
    #[error(
        r#"missing forward slash :: expected <number>/<unit> :: "10/s", "100/min", "1000/hour", ..."#
    )]
    MissingForwardSlash,

    #[error("invalid limit :: {0} :: expected <number>/<unit>")]
    InvalidLimit(#[from] std::num::ParseIntError),

    #[error(r#"invalid unit :: expected "s", "m", "h", "sec", "min", "hr", "second", "minute", "hour", "seconds", "minutes", "hours""#)]
    InvalidUnit,
}
