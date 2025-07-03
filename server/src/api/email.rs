use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use boxer::{Boxer, Context};
use cache::DashCache;
use data_access::DataAccess;
use email::Email;
use extra::json_error_response;
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
pub struct CheckEmailAvailabilityParams {
    pub email: String,
}

#[tracing::instrument(fields(?email), skip_all, ret)]
pub async fn check_email_availability(
    State(AppState { data_access, .. }): State<AppState>,
    Query(CheckEmailAvailabilityParams { email }): Query<CheckEmailAvailabilityParams>,
) -> Result<StatusCode, CheckEmailAvailabilityError> {
    let email = Email::try_from(email).map_err(CheckEmailAvailabilityError::InvalidParams)?;

    match email_exists(&data_access, &email)
        .await
        .context("check email availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

pub async fn email_exists(data_access: &DataAccess, email: &Email) -> Result<bool, sqlx::Error> {
    #[derive(Debug, Clone)]
    struct Row {
        user_id: i64,
    }

    let row = data_access
        .read(
            |pool| {
                sqlx::query_as!(
                    Row,
                    r#"SELECT id as "user_id!" FROM users WHERE email = ? LIMIT 1"#,
                    email
                )
                .fetch_optional(pool)
            },
            "email_exists",
            email.clone(),
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
pub enum CheckEmailAvailabilityError {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0:?}")]
    Internal(#[from] Boxer),
}

impl IntoResponse for CheckEmailAvailabilityError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CheckEmailAvailabilityError::InvalidParams(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::BAD_REQUEST, Json(json_error_response(self))).into_response()
            }
            CheckEmailAvailabilityError::Internal(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
