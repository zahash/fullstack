use axum::{
    http::StatusCode,
    routing::{MethodRouter, get},
};
use axum_macros::debug_handler;

use crate::AppState;

pub const PATH: &str = "/heartbeat";

pub fn method_router() -> MethodRouter<AppState> {
    get(handler)
}

#[debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    operation_id = PATH,
    responses((status = 200, description = "heartbeat OK")),
    tag = "probe"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(ret))]
pub async fn handler() -> StatusCode {
    StatusCode::OK
}
