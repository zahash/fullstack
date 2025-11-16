pub mod assign;
pub mod revoke;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{MethodRouter, get};
use axum_macros::debug_handler;
use contextual::Context;
use http::StatusCode;

use crate::core::InsufficientPermissionsError;
use crate::{
    AppState,
    core::{Permission, Principal},
};

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
) -> Result<Json<Vec<Permission>>, Error> {
    principal
        .require_permission::<Error>(&pool, "get:/permissions")
        .await?;

    let permissions = principal
        .permissions(&pool)
        .await
        .context("get permissions")?;

    Ok(Json(permissions))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InsufficientPermissions(#[from] InsufficientPermissionsError),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InsufficientPermissions(err) => err.into_response(),
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
