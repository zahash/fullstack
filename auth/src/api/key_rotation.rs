use axum::{
    Form,
    extract::State,
    response::IntoResponse,
    routing::{MethodRouter, post},
};
use http::StatusCode;
use serde::Deserialize;

use crate::{
    AppState,
    core::{InsufficientPermissionsError, Principal},
};

pub const PATH: &str = "/rotate-key";

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = key_rotation::RequestBody))]
#[derive(Deserialize)]
pub struct RequestBody {
    #[cfg_attr(feature = "openapi", schema(example = "HMAC"))]
    pub key: String,
}

pub fn method_router() -> MethodRouter<AppState> {
    post(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = PATH,
    operation_id = PATH,
    request_body(
        content = RequestBody,
        content_type = "application/x-www-form-urlencoded",
    ),
    responses(
        (status = 200, description = "Successfull Key Rotation"),
        (status = 401, description = "Invalid credentials", body = extra::ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = extra::ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "secrets"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%principal), skip_all, ret))]
pub async fn handler(
    State(AppState { pool, secrets, .. }): State<AppState>,
    principal: Principal,
    Form(RequestBody { key }): Form<RequestBody>,
) -> Result<StatusCode, Error> {
    principal
        .require_permission::<Error>(&pool, "post:/rotate-key")
        .await?;

    secrets.reset(&key)?;
    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InsufficientPermissions(#[from] InsufficientPermissionsError),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InsufficientPermissions(err) => err.into_response(),
            Error::Io(_) | Error::Sqlx(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
