mod error;
mod login;
mod private;
mod request_id;
mod session_id;
mod signup;
mod user_id;

use std::{net::SocketAddr, usize};

use anyhow::Context;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use request_id::RequestId;
use sqlx::SqlitePool;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};
use tokio::net::TcpListener;

use login::login;
use private::private;
use signup::{check_username_availability, signup};
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

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
        .with_state(AppState { pool })
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
        ))
        .layer(axum::middleware::from_fn(
            |request: Request<Body>, next: Next| async {
                let response = next.run(request).await;
                let status = response.status();

                match status.is_server_error() {
                    false => response,
                    true => {
                        // Log and capture the error details without exposing them to the client
                        // Avoid leaking sensitive internal information in the response body for 5xx errors
                        match to_bytes(response.into_body(), usize::MAX).await {
                            Ok(content) if !content.is_empty() => tracing::error!("{:?}", content),
                            Err(e) => tracing::error!(
                                "unable to convert INTERNAL_SERVER_ERROR response body to bytes :: {:?}",
                                e
                            ),
                            _ => {}
                        }
                        status.into_response()
                    }
                }
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

#[debug_handler]
#[tracing::instrument(ret)]
async fn health() -> (StatusCode, Json<System>) {
    (
        StatusCode::OK,
        Json(System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        )),
    )
}
