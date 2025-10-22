#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "axum", derive(serde::Serialize))]
#[derive(Debug, Clone)]
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

#[cfg(feature = "axum")]
impl extra::ErrorKind for PermissionError {
    fn kind(&self) -> &'static str {
        match self {
            PermissionError::InsufficientPermissionsError => "auth.permissions",
            PermissionError::Sqlx(_) => "auth.sqlx",
        }
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for PermissionError {
    fn into_response(self) -> axum::response::Response {
        match self {
            PermissionError::InsufficientPermissionsError => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (
                    axum::http::StatusCode::FORBIDDEN,
                    axum::Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
            PermissionError::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
