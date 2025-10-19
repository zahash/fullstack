use std::time::Instant;

use axum::{body::Body, http::Request, http::Response, middleware::Next};

pub async fn latency_ms(request: Request<Body>, next: Next) -> Response<Body> {
    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    tracing::info!("request processed in {} ms", duration.as_millis());
    response
}
