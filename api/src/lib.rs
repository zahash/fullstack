mod access_token;
mod check;
mod health;
mod login;
mod logout;
mod private;
mod signup;

use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use server_core::{
    AppState, RateLimiter, mw_client_ip, mw_handle_leaked_5xx, mw_rate_limiter, span,
};

use health::{health, sysinfo};
use login::login;
use logout::logout;
use private::private;
use signup::signup;

pub fn server(state: AppState) -> Router {
    Router::new()
        .nest(
            "/check",
            Router::new()
                .route("/username-availability", get(check::username_availability))
                .route("/email-availability", get(check::email_availability))
                .route("/access-token", get(check::access_token)),
        )
        .route("/health", get(health))
        .route("/sysinfo", get(sysinfo))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/access-token", post(access_token::generate))
        .route("/private", get(private))
        .with_state(state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(mw_handle_leaked_5xx))
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(PropagateRequestIdLayer::x_request_id())
                .layer(from_fn(mw_client_ip))
                .layer(TraceLayer::new_for_http().make_span_with(span))
                .layer(from_fn_with_state(state.clone(), mw_rate_limiter)),
        )
}

#[derive(Debug)]
pub struct ServerOpts {
    pub database_url: String,
    pub port: u16,
    pub ui_dir: PathBuf,
    // pub smtp: SMTPConfig,
}

#[derive(Debug)]
pub struct SMTPConfig {
    pub relay: String,
    pub username: String,
    pub password: String,
}

pub async fn run(opts: ServerOpts) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("{:?}", opts);

    let state = AppState {
        pool: SqlitePool::connect(&opts.database_url)
            .await
            .with_context(|| format!("connect database :: {}", opts.database_url))?,
        rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
        // mailer: Arc::new(()),
        // mailer: Arc::new(
        //     SmtpTransport::relay(&opts.smtp.relay)?
        //         .credentials((opts.smtp.username, opts.smtp.password).into())
        //         .build(),
        // ),
    };

    let app = server(state)
        .fallback_service(ServeDir::new(&opts.ui_dir))
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind :: {}", addr))?;
    tracing::info!(
        "listening on {}",
        listener.local_addr().context("local_addr")?
    );
    axum::serve(listener, app).await.context("axum::serve")?;
    Ok(())
}
