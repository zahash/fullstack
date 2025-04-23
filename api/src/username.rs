use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use server_core::{AppState, Context, InternalError, Username, error};
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct CheckUsernameAvailabilityParams {
    pub username: String,
}

#[tracing::instrument(fields(?username), skip_all, ret)]
pub async fn check_username_availability(
    State(AppState { pool, .. }): State<AppState>,
    Query(CheckUsernameAvailabilityParams { username }): Query<CheckUsernameAvailabilityParams>,
) -> Result<StatusCode, CheckUsernameAvailabilityError> {
    let username =
        Username::try_from(username).map_err(CheckUsernameAvailabilityError::InvalidParams)?;

    match username_exists(&pool, &username)
        .await
        .context("check username availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

pub async fn username_exists(pool: &SqlitePool, username: &Username) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = ? LIMIT 1) as username_exists",
        username
    )
    .fetch_one(pool)
    .await?
    .username_exists
    {
        0 => Ok(false),
        _ => Ok(true),
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
