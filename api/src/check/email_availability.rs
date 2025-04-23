use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::check::email_exists;

use server_core::{AppState, Context, Email, InternalError, error};

#[derive(Deserialize)]
pub struct Params {
    pub email: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[tracing::instrument(fields(?email), skip_all, ret)]
pub async fn email_availability(
    State(AppState { pool, .. }): State<AppState>,
    Query(Params { email }): Query<Params>,
) -> Result<StatusCode, Error> {
    let email = Email::try_from(email).map_err(Error::InvalidParams)?;

    match email_exists(&pool, &email)
        .await
        .context("check email availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InvalidParams(err) => {
                tracing::info!("{:?}", err);
                (StatusCode::BAD_REQUEST, error(err)).into_response()
            }
            Error::Internal(err) => err.into_response(),
        }
    }
}
