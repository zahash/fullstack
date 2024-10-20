use std::net::IpAddr;

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::{client_ip, AppState};

pub async fn mw_client_ip(mut request: Request<Body>, next: Next) -> Response<Body> {
    let ip = client_ip(&request);
    request.extensions_mut().insert(ip);
    next.run(request).await
}

pub async fn mw_rate_limiter(
    State(AppState { rate_limiter, .. }): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    if let Some(client_ip) = request
        .extensions()
        .get::<Option<IpAddr>>()
        .copied()
        .flatten()
        .or_else(|| {
            tracing::warn!("unable to get client_ip while rate limiting");
            None
        })
    {
        if rate_limiter.is_too_many(client_ip) {
            tracing::warn!("rate limited {}", client_ip);
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }
    }

    next.run(request).await
}

/// usually 5xx errors with internal details are handled
/// but under unforseen circumstances they leak to the client
/// this is the last line of defense to catch them
pub async fn mw_handle_leaked_5xx(request: Request<Body>, next: Next) -> Response<Body> {
    let response = next.run(request).await;
    let status = response.status();

    if status.is_server_error() {
        // Log and capture the error details without exposing them to the client
        match to_bytes(response.into_body(), usize::MAX).await {
            Ok(content) if !content.is_empty() => tracing::error!("{:?}", content),
            Err(e) => tracing::error!(
                "unable to convert INTERNAL_SERVER_ERROR response body to bytes :: {:?}",
                e
            ),
            _ => {}
        }

        return status.into_response();
    }

    response
}
