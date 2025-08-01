pub struct Permissions(pub Vec<Permission>);

#[derive(Debug, Clone)]
pub struct Permission {
    pub id: i64,
    pub permission: String,
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
