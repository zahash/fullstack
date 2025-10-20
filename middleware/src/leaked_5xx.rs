use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
    response::IntoResponse,
};

/// usually 5xx errors with internal details are handled
/// but under unforseen circumstances they leak to the client
/// this is the last line of defense to catch them
pub async fn handle_leaked_5xx(request: Request<Body>, next: Next) -> Response<Body> {
    let response = next.run(request).await;
    let status = response.status();

    if status.is_server_error() {
        // Log and capture the error details without exposing them to the client
        match axum::body::to_bytes(response.into_body(), usize::MAX).await {
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
