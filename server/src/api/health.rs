use axum::http::StatusCode;
use axum_macros::debug_handler;

pub const PATH: &str = "/health";

#[debug_handler]
#[tracing::instrument(ret)]
pub async fn handler() -> StatusCode {
    StatusCode::OK
}
