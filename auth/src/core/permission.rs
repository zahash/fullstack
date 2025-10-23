use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::Serialize;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct Permission {
    #[cfg_attr(feature = "openapi", schema(examples(1)))]
    pub id: i64,

    #[cfg_attr(feature = "openapi", schema(examples("post:/access-token/generate")))]
    pub permission: String,

    #[cfg_attr(feature = "openapi", schema(examples("Generate a new access token")))]
    pub description: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum PermissionError {
    #[error("insufficient permissions")]
    InsufficientPermissionsError,

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for PermissionError {
    fn kind(&self) -> &'static str {
        match self {
            PermissionError::InsufficientPermissionsError => "auth.permissions",
            PermissionError::Sqlx(_) => "auth.sqlx",
        }
    }
}

impl IntoResponse for PermissionError {
    fn into_response(self) -> Response {
        match self {
            PermissionError::InsufficientPermissionsError => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (
                    StatusCode::FORBIDDEN,
                    Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
            PermissionError::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
