#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "axum", derive(serde::Serialize))]
#[derive(Debug)]
pub struct Permissions(pub Vec<Permission>);

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
#[error("insufficient permissions")]
pub struct InsufficientPermissionsError;

impl Permissions {
    pub fn contains(&self, permission: &str) -> bool {
        self.0
            .iter()
            .map(|p| &p.permission)
            .any(|s| s == permission)
    }

    pub fn require(&self, permission: &str) -> Result<(), InsufficientPermissionsError> {
        match self.contains(permission) {
            true => Ok(()),
            false => Err(InsufficientPermissionsError),
        }
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Permissions {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

#[cfg(feature = "axum")]
impl extra::ErrorKind for InsufficientPermissionsError {
    fn kind(&self) -> &'static str {
        "auth.permissions"
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for InsufficientPermissionsError {
    fn into_response(self) -> axum::response::Response {
        #[cfg(feature = "tracing")]
        tracing::info!("{:?}", self);
        (
            axum::http::StatusCode::FORBIDDEN,
            axum::Json(extra::ErrorResponse::from(self)),
        )
            .into_response()
    }
}
