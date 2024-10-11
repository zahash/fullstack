mod error;
mod extractor;
mod login;
mod private;
mod signup;
mod types;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{
    extract::Extension,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

use error::AppError;
use login::login;
use private::private;
use signup::signup;
use tower_http::{services::ServeDir, trace::TraceLayer};
use types::TraceId;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt().init();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL")?;
    tracing::info!(DATABASE_URL = %database_url);
    let port: u16 = std::env::var("PORT")
        .context("PORT")?
        .parse()
        .context("parse PORT")?;
    let ui = std::env::var("UI").context("UI")?;
    tracing::info!(UI = %ui);

    let pool = SqlitePool::connect(&database_url)
        .await
        .context(format!("connect database :: {}", database_url))?;

    let app = Router::new()
        .nest_service("/", ServeDir::new(ui))
        .route("/health", get(health))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/private", get(private))
        .layer(Extension(pool))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let trace_id = TraceId::new();
                tracing::info_span!(
                    "request",
                    trace_id = %trace_id,
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)
        .await
        .context(format!("bind :: {}", addr))?;
    tracing::info!(
        "listening on {}",
        listener.local_addr().context("local_addr")?
    );
    axum::serve(listener, app).await.context("axum::serve")?;
    Ok(())
}

async fn health() -> StatusCode {
    StatusCode::OK
}
