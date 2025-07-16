use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use contextual::Context;
use dashcache::DashCache;
use data_access::DataAccess;
use extra::json_error_response;
use serde::Deserialize;

use tag::Tag;
use validation::validate_username;

use crate::AppState;

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
) -> Result<bool, data_access::Error> {
    #[derive(Debug, Clone)]
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
                Some(row) => vec![Tag {
                    table: "users",
                    primary_key: Some(row.user_id),
                }],
                None => vec![Tag {
                    table: "users",
                    primary_key: None,
                }],
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
    DataAccess(#[from] contextual::Error<data_access::Error>),
}

impl IntoResponse for CheckUsernameAvailabilityError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CheckUsernameAvailabilityError::InvalidParams(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::BAD_REQUEST, Json(json_error_response(self))).into_response()
            }
            CheckUsernameAvailabilityError::DataAccess(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
