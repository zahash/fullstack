use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::check::username_exists;

use server_core::{AppState, Context, InternalError, Username, error};

#[derive(Deserialize)]
pub struct Params {
    pub username: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[tracing::instrument(fields(?username), skip_all, ret)]
pub async fn username_availability(
    State(AppState { pool, .. }): State<AppState>,
    Query(Params { username }): Query<Params>,
) -> Result<StatusCode, Error> {
    let username = Username::try_from(username).map_err(Error::InvalidParams)?;

    match username_exists(&pool, &username)
        .await
        .context("check username availability")?
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
