use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use contextual::Context;
use email::Email;
use extra::json_error_response;
use serde::Deserialize;

use crate::AppState;

pub const PATH: &str = "/check/email-availability";

#[derive(Deserialize)]
pub struct CheckEmailAvailabilityParams {
    pub email: String,
}

#[tracing::instrument(fields(%email), skip_all, ret)]
pub async fn handler(
    State(AppState { data_access, .. }): State<AppState>,
    Query(CheckEmailAvailabilityParams { email }): Query<CheckEmailAvailabilityParams>,
) -> Result<StatusCode, CheckEmailAvailabilityError> {
    let email = Email::try_from(email).map_err(CheckEmailAvailabilityError::InvalidParams)?;

    match super::exists(&data_access, &email)
        .await
        .context("check email availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CheckEmailAvailabilityError {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0}")]
    DataAccess(#[from] contextual::Error<data_access::Error>),
}

impl IntoResponse for CheckEmailAvailabilityError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CheckEmailAvailabilityError::InvalidParams(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::BAD_REQUEST, Json(json_error_response(self))).into_response()
            }
            CheckEmailAvailabilityError::DataAccess(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
