use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_macros::debug_handler;

use crate::{
    AppState,
    error::InternalError,
    types::{
        AccessToken, AccessTokenExtractionError, AccessTokenInfoError, AccessTokenValiationError,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    AccessTokenExtraction(#[from] AccessTokenExtractionError),

    #[error("{0}")]
    AccessTokenInfo(#[from] AccessTokenInfoError),

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValiationError),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[debug_handler]
pub async fn access_token(
    State(AppState { pool, .. }): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, Error> {
    AccessToken::try_from(&headers)?
        .info(&pool)
        .await?
        .validate()?;

    Ok(StatusCode::OK)
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::AccessTokenExtraction(err) => err.into_response(),
            Error::AccessTokenInfo(err) => err.into_response(),
            Error::AccessTokenValidation(err) => err.into_response(),
            Error::Internal(err) => err.into_response(),
        }
    }
}
