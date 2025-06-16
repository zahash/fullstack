use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use cache::DashCache;
use data_access::DataAccess;
use serde::Deserialize;

use server_core::{AppState, Context, InternalError, error};
use validation::validate_username;

#[derive(Deserialize)]
pub struct CheckUsernameAvailabilityParams {
    pub username: String,
}

#[tracing::instrument(fields(?username), skip_all, ret)]
pub async fn check_username_availability(
    State(AppState { data_access, .. }): State<AppState>,
    Query(CheckUsernameAvailabilityParams { username }): Query<CheckUsernameAvailabilityParams>,
) -> Result<StatusCode, CheckUsernameAvailabilityError> {
    let username =
        validate_username(username).map_err(CheckUsernameAvailabilityError::InvalidParams)?;

    match username_exists(&data_access, &username)
        .await
        .context("check username availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

pub async fn username_exists(
    data_access: &DataAccess,
    username: &str,
) -> Result<bool, sqlx::Error> {
    #[derive(Clone)]
    struct Row {
        user_id: i64,
    }

    let row = data_access
        .read(
            |pool| {
                sqlx::query_as!(
                    Row,
                    r#"SELECT id as "user_id!" FROM users WHERE username = ? LIMIT 1"#,
                    username
                )
                .fetch_optional(pool)
            },
            "username_exists",
            username.to_string(),
            |value| match value {
                Some(row) => vec![Box::new(format!("users:{}", row.user_id))],
                None => vec![Box::new("users")],
            },
            DashCache::new,
        )
        .await?;

    match row {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CheckUsernameAvailabilityError {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

impl IntoResponse for CheckUsernameAvailabilityError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CheckUsernameAvailabilityError::InvalidParams(err) => {
                tracing::info!("{:?}", err);
                (StatusCode::BAD_REQUEST, error(err)).into_response()
            }
            CheckUsernameAvailabilityError::Internal(err) => err.into_response(),
        }
    }
}
