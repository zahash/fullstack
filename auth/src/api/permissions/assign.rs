use axum::{
    Json,
    extract::State,
    http::StatusCode,
    routing::{MethodRouter, post},
};
use contextual::Context;
use extra::ErrorResponse;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    AppState,
    core::{InsufficientPermissionsError, Principal},
};

// TODO: mark this as admin endpoint. maybe using tags

pub const PATH: &str = "/permissions";

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
        (status = 201, description = "Permission assigned successfully"),
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
        .require_permission::<Error>(&pool, "post:/permissions")
        .await?;

    // The Assigner must have the requested permission themselves first
    // before they assign it to others
    principal
        .require_permission::<Error>(&pool, &request_body.permission)
        .await?;

    let (assigner_type, assigner_id) = match principal {
        Principal::Session(info) => ("user", info.user_id),
        Principal::AccessToken(info) => ("access_token", info.id),
        Principal::Basic(info) => ("user", info.user_id),
    };

    let mut tx = pool
        .begin()
        .await
        .context("begin transaction :: assign permission")?;

    let (assignee_type, assignee_id, permission_id) = match request_body.assignee {
        Assignee::User { username } => match sqlx::query!(
            r#"
            INSERT INTO user_permissions (user_id, permission_id)

            SELECT u.id, p.id
            FROM users u
            INNER JOIN user_permissions up ON up.user_id = u.id
            INNER JOIN permissions p ON p.id = up.permission_id

            WHERE u.username = ? AND p.permission = ?

            ON CONFLICT(user_id, permission_id) DO NOTHING
            RETURNING user_id, permission_id
            "#,
            username,
            request_body.permission
        )
        .fetch_optional(&mut *tx)
        .await
        .context("assign permission to user")?
        {
            None => return Err(Error::DoesNotExist),
            Some(record) => ("user", record.user_id, record.permission_id),
        },
        Assignee::AccessToken {
            username,
            token_name,
        } => match sqlx::query!(
            r#"
            INSERT INTO access_token_permissions (access_token_id, permission_id)

            SELECT a.id, p.id
            FROM access_tokens a
            INNER JOIN users u ON u.id = a.user_id
            INNER JOIN access_token_permissions ap ON ap.access_token_id = a.id
            INNER JOIN permissions p ON p.id = ap.permission_id

            WHERE u.username = ? AND a.name = ? AND p.permission = ?

            ON CONFLICT(access_token_id, permission_id) DO NOTHING
            RETURNING access_token_id, permission_id
            "#,
            username,
            token_name,
            request_body.permission
        )
        .fetch_optional(&mut *tx)
        .await
        .context("assign permission to access token")?
        {
            None => return Err(Error::DoesNotExist),
            Some(record) => ("access_token", record.access_token_id, record.permission_id),
        },
    };

    let now = OffsetDateTime::now_utc();
    sqlx::query!(
        r#"
        INSERT INTO permissions_audit_log
        (
            assigner_type,
            assigner_id,
            assignee_type,
            assignee_id,
            permission_id,
            action,
            datetime
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        assigner_type,
        assigner_id,
        assignee_type,
        assignee_id,
        permission_id,
        "assign",
        now
    )
    .execute(&mut *tx)
    .await
    .context("write permission audit log")?;

    tx.commit()
        .await
        .context("commit transaction :: assign permission")?;

    #[cfg(feature = "tracing")]
    tracing::info!("permission assigned");

    Ok(StatusCode::CREATED)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InsufficientPermissions(#[from] InsufficientPermissionsError),

    #[error("either the assignee or the permission does not exist")]
    DoesNotExist,

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::InsufficientPermissions(e) => e.kind(),
            Error::DoesNotExist => "does_not_exist",
            Error::Sqlx(_) => "sqlx",
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InsufficientPermissions(err) => err.into_response(),
            Error::DoesNotExist => {
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
