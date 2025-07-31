use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use contextual::Context;
use extra::json_error_response;
use serde::Deserialize;

use validation::validate_username;

use crate::AppState;

pub const PATH: &str = "/check/username-availability";

#[derive(Deserialize)]
pub struct CheckUsernameAvailabilityParams {
    pub username: String,
}

#[tracing::instrument(fields(%username), skip_all, ret)]
pub async fn handler(
    State(AppState { data_access, .. }): State<AppState>,
    Query(CheckUsernameAvailabilityParams { username }): Query<CheckUsernameAvailabilityParams>,
) -> Result<StatusCode, CheckUsernameAvailabilityError> {
    let username =
        validate_username(username).map_err(CheckUsernameAvailabilityError::InvalidParams)?;

    match super::exists(&data_access, &username)
        .await
        .context("check username availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CheckUsernameAvailabilityError {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0}")]
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
