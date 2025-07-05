mod api;
mod middleware;
mod span;

use std::net::SocketAddr;

use axum::{
    Router,
    extract::FromRef,
    middleware::from_fn,
    routing::{get, post},
};
use boxer::{Boxer, Context};
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
#[derive(Debug)]
pub struct RateLimiterConfig {
    pub limit: usize,
    pub interval: std::time::Duration,
}

#[cfg(feature = "smtp")]
#[derive(Debug)]
pub struct SMTPConfig {
    pub relay: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct AppState {
    pub data_access: DataAccess,
}

impl FromRef<AppState> for DataAccess {
    fn from_ref(input: &AppState) -> Self {
        input.data_access.clone()
    }
}

pub fn server(
    data_access: DataAccess,
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

    Router::new()
        .nest(
            "/check",
            Router::new()
                .route("/username-availability", get(check_username_availability))
                .route("/email-availability", get(check_email_availability))
                .route("/access-token", get(check_access_token)),
        )
        .route("/health", get(health))
        .route("/sysinfo", get(sysinfo))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/access-token", post(generate_access_token))
        .route("/private", get(private))
        .with_state(AppState { data_access })
        .layer(middleware)
}

pub async fn serve(opts: ServerOpts) -> Result<(), Boxer> {
    tracing::info!("{:?}", opts);

    let data_access = DataAccess::new(
        SqlitePool::connect(&opts.database_url)
            .await
            .context(format!("connect database :: {}", opts.database_url))?,
    );

    #[cfg(feature = "rate-limit")]
    let rate_limiter =
        crate::middleware::RateLimiter::new(opts.rate_limiter.limit, opts.rate_limiter.interval);

    let server = server(
        data_access,
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
    axum::serve(listener, app).await.context("axum::serve")
}
