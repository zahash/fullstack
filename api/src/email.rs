use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use server_core::{AppState, Context, Email, InternalError, error};
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct CheckEmailAvailabilityParams {
    pub email: String,
}

#[tracing::instrument(fields(?email), skip_all, ret)]
pub async fn check_email_availability(
    State(AppState { pool, .. }): State<AppState>,
    Query(CheckEmailAvailabilityParams { email }): Query<CheckEmailAvailabilityParams>,
) -> Result<StatusCode, CheckEmailAvailabilityError> {
    let email = Email::try_from(email).map_err(CheckEmailAvailabilityError::InvalidParams)?;

    match email_exists(&pool, &email)
        .await
        .context("check email availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

pub async fn email_exists(pool: &SqlitePool, email: &Email) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = ? LIMIT 1) as email_exists",
        email
    )
    .fetch_one(pool)
    .await?
    .email_exists
    {
        0 => Ok(false),
        _ => Ok(true),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CheckEmailAvailabilityError {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

impl IntoResponse for CheckEmailAvailabilityError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CheckEmailAvailabilityError::InvalidParams(err) => {
                tracing::info!("{:?}", err);
                (StatusCode::BAD_REQUEST, error(err)).into_response()
            }
            CheckEmailAvailabilityError::Internal(err) => err.into_response(),
        }
    }
}
