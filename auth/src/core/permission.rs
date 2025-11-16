use axum::{
    Json,
    response::{IntoResponse, Response},
};
use contextual::Context;
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

pub trait Authorizable {
    async fn permissions(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Vec<Permission>, sqlx::Error>;

    /// has_permission by default fetches permissions and checks for a match.
    /// Implementors MAY override with a more efficient implementation (e.g. EXISTS query).
    async fn has_permission(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        let permissions = self.permissions(pool).await?;
        Ok(permissions.iter().any(|p| p.permission == permission))
    }

    async fn require_permission<E>(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        permission: &str,
    ) -> Result<(), E>
    where
        E: std::error::Error
            + From<InsufficientPermissionsError>
            + From<contextual::Error<sqlx::Error>>,
    {
        match self
            .has_permission(pool, permission)
            .await
            .context(format!("require_permission `{permission}`"))
        {
            Ok(true) => Ok(()),
            Ok(false) => Err(InsufficientPermissionsError.into()),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("insufficient permissions")]
pub struct InsufficientPermissionsError;

impl extra::ErrorKind for InsufficientPermissionsError {
    fn kind(&self) -> &'static str {
        "auth.permissions"
    }
}

impl IntoResponse for InsufficientPermissionsError {
    fn into_response(self) -> Response {
        #[cfg(feature = "tracing")]
        tracing::info!("{:?}", self);

        (
            StatusCode::FORBIDDEN,
            Json(extra::ErrorResponse::from(self)),
        )
            .into_response()
    }
}
