mod api;
mod core;
mod secrets;

#[cfg(feature = "tracing")]
mod span;

#[cfg(feature = "smtp")]
mod smtp;

use std::net::SocketAddr;

use axum::{Router, extract::FromRef, middleware::from_fn};
use contextual::Context;
use http::HeaderName;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

use crate::secrets::Secrets;

#[derive(Debug)]
pub struct ServerOpts {
    pub database: DatabaseConfig,
    pub secrets_dir: std::path::PathBuf,

    #[cfg(feature = "rate-limit")]
    pub rate_limiter: RateLimiterConfig,

    #[cfg(feature = "serve-dir")]
    pub serve_dir: std::path::PathBuf,

    #[cfg(feature = "smtp")]
    pub smtp: SmtpConfig,
}

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
}

#[cfg(feature = "rate-limit")]
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub limit: usize,
    pub interval: std::time::Duration,
}

#[cfg(feature = "smtp")]
#[derive(Debug)]
pub struct SmtpConfig {
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
    pub secrets: Secrets,

    #[cfg(feature = "smtp")]
    pub smtp: crate::smtp::Smtp,
}

pub async fn router(opts: ServerOpts) -> Result<Router, ServerError> {
    use crate::api::{
        access_token, email, heartbeat, key_rotation, login, logout, permissions, private, signup,
        sysinfo, username,
    };

    let router = Router::new()
        .route(
            access_token::generate::PATH,
            access_token::generate::method_router(),
        )
        .route(
            access_token::verify::PATH,
            access_token::verify::method_router(),
        )
        .route(
            email::check_availability::PATH,
            email::check_availability::method_router(),
        )
        .route(heartbeat::PATH, heartbeat::method_router())
        .route(key_rotation::PATH, key_rotation::method_router())
        .route(login::PATH, login::method_router())
        .route(logout::PATH, logout::method_router())
        .route(permissions::PATH, permissions::method_router())
        .route(
            permissions::assign::PATH,
            permissions::assign::method_router(),
        )
        .route(private::PATH, private::method_router())
        .route(signup::PATH, signup::method_router())
        .route(sysinfo::PATH, sysinfo::method_router())
        .route(
            username::check_availability::PATH,
            username::check_availability::method_router(),
        );

    #[cfg(feature = "smtp")]
    let router = router
        .route(
            email::initiate_verification::PATH,
            email::initiate_verification::method_router(),
        )
        .route(
            email::verify_email::PATH,
            email::verify_email::method_router(),
        );

    #[cfg(feature = "openapi")]
    let router = router.route(
        api::OPEN_API_DOCS_PATH,
        axum::routing::get(axum::Json(api::openapi())),
    );

    #[cfg(feature = "serve-dir")]
    let router = router.fallback_service(tower_http::services::ServeDir::new(&opts.serve_dir));

    const X_TRACE_ID: HeaderName = HeaderName::from_static("x-trace-id");
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(X_TRACE_ID, MakeRequestUuid))
        .layer(PropagateRequestIdLayer::new(X_TRACE_ID));

    #[cfg(feature = "tracing")]
    let middleware = middleware
        .layer(tower_http::trace::TraceLayer::new_for_http().make_span_with(span::span))
        .layer(from_fn(middleware::latency_ms));

    let middleware = middleware.layer(from_fn(middleware::handle_leaked_5xx));

    #[cfg(feature = "rate-limit")]
    let middleware = middleware.layer(axum::middleware::from_fn_with_state(
        std::sync::Arc::new(opts.rate_limiter.into()),
        middleware::rate_limiter,
    ));

    let router = router.layer(middleware);

    let router = router.with_state(AppState {
        pool: opts
            .database
            .pool()
            .await
            .context(format!("connect database :: {}", opts.database.url))?,
        secrets: Secrets::new(opts.secrets_dir),
        #[cfg(feature = "smtp")]
        smtp: crate::smtp::Smtp::try_from(opts.smtp)?,
    });

    Ok(router)
}

/// Returns the local address that the listener is bound to.
/// This can be useful, for example, when binding to port 0 to figure out which port was actually bound.
pub async fn serve(server: Router, port: u16) -> Result<SocketAddr, ServerError> {
    #[cfg(feature = "client-ip")]
    let app = server.into_make_service_with_connect_info::<SocketAddr>();

    #[cfg(not(feature = "client-ip"))]
    let app = server.into_make_service();

    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .context("bind")?;

    let local_addr = listener.local_addr().context("local_addr")?;

    #[cfg(feature = "tracing")]
    tracing::info!("listening on {}", local_addr);

    axum::serve(listener, app).await.context("axum::serve")?;
    Ok(local_addr)
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),

    #[cfg(feature = "smtp")]
    #[error("{0}")]
    SmtpInitialization(#[from] SmtpInitializationError),

    #[error("{0}")]
    Io(#[from] contextual::Error<std::io::Error>),
}

#[cfg(feature = "smtp")]
#[derive(thiserror::Error, Debug)]
pub enum SmtpInitializationError {
    #[cfg(feature = "smtp")]
    #[error("{0}")]
    SmtpTransport(#[from] contextual::Error<lettre::transport::smtp::Error>),

    #[cfg(feature = "smtp")]
    #[error("{0}")]
    Tera(#[from] contextual::Error<tera::Error>),
}

impl FromRef<AppState> for sqlx::Pool<sqlx::Sqlite> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.pool.clone()
    }
}

impl DatabaseConfig {
    pub async fn pool(&self) -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
        sqlx::Pool::<sqlx::Sqlite>::connect(&self.url).await
    }
}

#[cfg(feature = "rate-limit")]
impl From<RateLimiterConfig> for middleware::RateLimiter {
    fn from(config: RateLimiterConfig) -> Self {
        Self::new(config.limit, config.interval)
    }
}

#[cfg(feature = "smtp")]
impl TryFrom<SmtpConfig> for crate::smtp::Smtp {
    type Error = SmtpInitializationError;

    fn try_from(config: SmtpConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            transport: {
                #[cfg(not(feature = "smtp--no-tls"))]
                let mut transport =
                    lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(
                        &config.relay,
                    )
                    .context("smtp relay")?;

                #[cfg(feature = "smtp--no-tls")]
                let mut transport =
                    lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(
                        &config.relay,
                    );

                if let (Some(username), Some(password)) = (config.username, config.password) {
                    use lettre::transport::smtp::authentication::Credentials;
                    transport = transport.credentials(Credentials::new(username, password));
                }

                if let Some(port) = config.port {
                    transport = transport.port(port);
                }

                transport.build()
            },
            senders: std::sync::Arc::new(crate::smtp::SmtpSenders::new(config.senders_dir)),
            tera: {
                let glob = config.templates_dir.join("*.html");
                let glob_str = glob.to_string_lossy().to_string();
                let tera = tera::Tera::new(&glob_str).context("initialize Tera")?;
                std::sync::Arc::new(tera)
            },
        })
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
