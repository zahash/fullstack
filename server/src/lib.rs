mod api;
mod middleware;
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
use data_access::DataAccess;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    api::{
        check_access_token, check_email_availability, check_username_availability,
        generate_access_token, health, login, logout, private, signup, sysinfo,
    },
    middleware::{mw_client_ip, mw_handle_leaked_5xx},
    span::span,
};

#[derive(Debug)]
pub struct ServerOpts {
    pub database_url: String,
    pub port: u16,

    #[cfg(feature = "rate-limit")]
    pub rate_limiter: RateLimiterConfig,

    #[cfg(feature = "ui")]
    pub ui_dir: std::path::PathBuf,

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
    pub data_access: DataAccess,

    #[cfg(feature = "smtp")]
    pub smtp: crate::smtp::Smtp,
}

pub fn server(
    data_access: DataAccess,
    #[cfg(feature = "smtp")] smtp: crate::smtp::Smtp,
    #[cfg(feature = "rate-limit")] rate_limiter: crate::middleware::RateLimiter,
) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(from_fn(mw_handle_leaked_5xx))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(from_fn(mw_client_ip))
        .layer(TraceLayer::new_for_http().make_span_with(span));

    #[cfg(feature = "rate-limit")]
    let middleware = middleware.layer(axum::middleware::from_fn_with_state(
        std::sync::Arc::new(rate_limiter),
        crate::middleware::mw_rate_limiter,
    ));

    let router = Router::new()
        .nest(
            "/check",
            Router::new()
                .route("/username-availability", get(check_username_availability))
                .route("/email-availability", get(check_email_availability))
                .route("/access-token", get(check_access_token)),
        )
        .route("/health", get(health))
        .route("/sysinfo", get(sysinfo))
        .route(signup::PATH, post(signup::handler))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/access-token", post(generate_access_token))
        .route("/private", get(private));

    #[cfg(feature = "smtp")]
    let router = router
        .nest(
            "/check",
            Router::new().route(
                "/email-verification-token",
                get(crate::api::check_email_verification_token),
            ),
        )
        .route(
            "/initiate-email-verification",
            get(crate::api::initiate_email_verification),
        );

    #[cfg(feature = "openapi")]
    let router = {
        use crate::api::OpenApiDoc;
        use axum::Json;
        use utoipa::OpenApi;

        router.route("/api-doc/openapi.json", get(Json(OpenApiDoc::openapi())))
    };

    #[cfg(feature = "scalar")]
    let router = {
        use crate::api::OpenApiDoc;
        use axum::response::Html;
        use utoipa::OpenApi;
        use utoipa_scalar::Scalar;

        router.route(
            "/scalar",
            get(Html(Scalar::new(OpenApiDoc::openapi()).to_html())),
        )
    };

    router
        .with_state(AppState {
            data_access,

            #[cfg(feature = "smtp")]
            smtp,
        })
        .layer(middleware)
}

pub async fn serve(opts: ServerOpts) -> Result<(), ServerError> {
    tracing::info!("{:?}", opts);

    let data_access = DataAccess::new(
        SqlitePool::connect(&opts.database_url)
            .await
            .context(format!("connect database :: {}", opts.database_url))?,
    );

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
        data_access,
        #[cfg(feature = "smtp")]
        smtp,
        #[cfg(feature = "rate-limit")]
        rate_limiter,
    );

    #[cfg(feature = "ui")]
    let server = server.fallback_service(tower_http::services::ServeDir::new(&opts.ui_dir));

    let app = server.into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let listener = TcpListener::bind(addr)
        .await
        .context(format!("bind :: {addr}"))?;
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

impl FromRef<AppState> for DataAccess {
    fn from_ref(input: &AppState) -> Self {
        input.data_access.clone()
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
