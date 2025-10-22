pub mod assign;

use authlib::{Permission, PermissionError, Principal};
use axum::Json;
use axum::extract::State;
use axum::routing::{MethodRouter, get};
use axum_macros::debug_handler;
use contextual::Context;

use crate::AppState;

pub const PATH: &str = "/permissions";

pub fn method_router() -> MethodRouter<AppState> {
    get(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    responses(
        (status = 200, description = "permissions", body = Vec<Permission>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error")
    ),
    tag = "permissions"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%principal), skip_all, ret))]
#[debug_handler]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    principal: Principal,
) -> Result<Json<Vec<Permission>>, PermissionError> {
    principal
        .require_permission(&pool, "get:/permissions")
        .await?;

    let permissions = principal
        .permissions(&pool)
        .await
        .context("get permissions")?;

    Ok(Json(permissions))
}
