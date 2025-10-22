use authlib::{PermissionError, Principal};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    routing::{MethodRouter, post},
};
use contextual::Context;
use extra::ErrorResponse;
use serde::Deserialize;

use crate::AppState;

pub const PATH: &str = "/permissions/assign";

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Deserialize)]
pub struct RequestBody {
    pub permission: String,
    pub assignee: Assignee,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assignee {
    AccessToken {
        #[cfg_attr(feature = "openapi", schema(examples("joe")))]
        username: String,

        #[cfg_attr(feature = "openapi", schema(examples("my-token")))]
        token_name: String,
    },
    User {
        #[cfg_attr(feature = "openapi", schema(examples("joe")))]
        username: String,
    },
}

pub fn method_router() -> MethodRouter<AppState> {
    post(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = PATH,
    request_body = RequestBody,
    responses(
        (status = 204, description = "Permission assigned successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Assignee not found")
    ),
    tag = "permissions"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%principal), skip_all, ret))]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    principal: Principal,
    Json(request_body): Json<RequestBody>,
) -> Result<StatusCode, Error> {
    principal
        .require_permission(&pool, "post:/permissions/assign")
        .await?;

    // The Assigner must have the requested permission themselves first
    // before they assign it to others
    principal
        .require_permission(&pool, &request_body.permission)
        .await?;

    let rows_affected = match request_body.assignee {
        Assignee::User { username } => sqlx::query!(
            r#"
            INSERT INTO user_permissions (user_id, permission_id)

            SELECT u.id, p.id
            FROM users u
            INNER JOIN user_permissions up ON up.user_id = u.id
            INNER JOIN permissions p ON p.id = up.permission_id

            WHERE u.username = ? AND p.permission = ?
            "#,
            username,
            request_body.permission
        )
        .execute(&pool)
        .await
        .context("assign permission to user")?
        .rows_affected(),
        Assignee::AccessToken {
            username,
            token_name,
        } => sqlx::query!(
            r#"
            INSERT INTO access_token_permissions (access_token_id, permission_id)

            SELECT a.id, p.id
            FROM access_tokens a
            INNER JOIN users u ON u.id = a.user_id
            INNER JOIN access_token_permissions ap ON ap.access_token_id = a.id
            INNER JOIN permissions p ON p.id = ap.permission_id

            WHERE u.username = ? AND a.name = ? AND p.permission = ?
            "#,
            username,
            token_name,
            request_body.permission
        )
        .execute(&pool)
        .await
        .context("assign permission to access token")?
        .rows_affected(),
    };

    if rows_affected == 0 {
        return Err(Error::UnAssigned);
    }

    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Permission(#[from] PermissionError),

    #[error("permission not assigned because either the assignee or the permission does not exist")]
    UnAssigned,

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::Permission(e) => e.kind(),
            Error::UnAssigned => "unassigned",
            Error::Sqlx(_) => "sqlx",
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Permission(err) => err.into_response(),
            Error::UnAssigned => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::NOT_FOUND, Json(ErrorResponse::from(self))).into_response()
            }
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
