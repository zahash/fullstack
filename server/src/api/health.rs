use axum::http::StatusCode;
use axum_macros::debug_handler;

pub const PATH: &str = "/health";

#[debug_handler]
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    operation_id = PATH,
    responses((status = 200, description = "health check OK")),
    tag = "probe"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(ret))]
pub async fn handler() -> StatusCode {
    StatusCode::OK
}
