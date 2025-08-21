pub mod assign;

use auth::{InsufficientPermissionsError, Permissions, Principal};
use axum::{extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use contextual::Context;
use http::StatusCode;

use crate::AppState;

pub const PATH: &str = "/permissions";

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    responses(
        (status = 200, description = "permissions", body = Permissions),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error")
    ),
    tag = "permissions"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(user_id = tracing::field::Empty), skip_all, ret))]
#[debug_handler]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    principal: Principal,
) -> Result<Permissions, Error> {
    #[cfg(feature = "tracing")]
    tracing::Span::current().record("user_id", tracing::field::display(principal.user_id()));

    let permissions = principal
        .permissions(&pool)
        .await
        .context("get permissions")?;
    permissions.require("get:/permissions")?;

    Ok(permissions)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Permission(#[from] InsufficientPermissionsError),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Permission(err) => err.into_response(),
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
