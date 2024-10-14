mod error;
mod extractor;
mod login;
mod private;
mod signup;
mod types;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
    middleware::Next,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

use login::login;
use private::private;
use signup::{check_username_availability, signup};
use tower_http::{services::ServeDir, trace::TraceLayer};
use types::RequestId;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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
        .route(
            "/check-username-availability",
            post(check_username_availability),
        )
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/private", get(private))
        .layer(Extension(pool))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = request
                    .extensions()
                    .get::<RequestId>()
                    .cloned()
                    .unwrap_or_else(|| {
                        tracing::warn!("unable to get RequestId extension when making span");
                        RequestId::unknown()
                    });

                tracing::info_span!(
                    "request",
                    request_id = %request_id,
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        .layer(axum::middleware::from_fn(
            |mut request: Request<Body>, next: Next| async {
                let request_id = RequestId::new();
                request.extensions_mut().insert(request_id);
                next.run(request).await
            },
        ));

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

#[tracing::instrument(ret)]
async fn health() -> StatusCode {
    StatusCode::OK
}
