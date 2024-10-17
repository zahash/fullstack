mod access_token;
mod error;
mod health;
mod login;
mod private;
mod request_id;
mod session_id;
mod signup;
mod token;
mod user_id;

use std::usize;

use axum::{
    body::{to_bytes, Body},
    http::Request,
    middleware::Next,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use health::health;
use request_id::RequestId;
use sqlx::SqlitePool;

use login::login;
use private::private;
use signup::{check_username_availability, signup};
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub fn ui(path: &str) -> Router {
    Router::<()>::new().nest_service("/", ServeDir::new(path))
}

pub fn server(pool: SqlitePool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/check-username-availability",
            post(check_username_availability),
        )
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/access-token", post(access_token::generate))
        .route("/private", get(private))
        .with_state(AppState { pool })
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| match request
                .extensions()
                .get::<RequestId>() {
                    Some(request_id) => tracing::info_span!(
                        "request",
                        "{} {} {}",
                        request_id,
                        request.method(),
                        request.uri(),
                    ),
                    None => tracing::info_span!(
                        "request",
                        "{} {}",
                        request.method(),
                        request.uri(),
                    ),
                }
            ),
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
        ))
}
