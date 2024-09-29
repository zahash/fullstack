use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use tracing::info_span;

use crate::{error::AppError, types::TraceId};

pub async fn trace_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let trace_id = TraceId::new();
    let span = info_span!("request", trace_id = %trace_id);
    let _enter = span.enter();
    req.extensions_mut().insert(trace_id);
    let response = next.run(req).await;
    Ok(response)
}
