use auth::{
    AccessToken, AccessTokenAuthorizationExtractionError, AccessTokenValidationError, Credentials,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use contextual::Context;
use extra::ErrorResponse;

use crate::AppState;

pub const PATH: &str = "/check/access-token";

#[tracing::instrument(skip_all, ret)]
pub async fn handler(
    State(AppState { data_access, .. }): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, Error> {
    let access_token =
        AccessToken::try_from_headers(&headers)?.ok_or_else(|| Error::AccessTokenHeaderNotFound)?;

    let info = access_token
        .info(&data_access)
        .await
        .context("AccessToken -> AccessTokenInfo")?
        .ok_or(Error::UnAssociatedAccessToken)?;

    tracing::info!(
        "user id = {}; access token name = {}",
        info.user_id,
        info.name
    );

    info.verify()?;

    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    AccessTokenAuthorizationExtractionError(#[from] AccessTokenAuthorizationExtractionError),

    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenHeaderNotFound,

    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValidationError),

    #[error("{0}")]
    DataAccess(#[from] contextual::Error<data_access::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::AccessTokenAuthorizationExtractionError(err) => err.into_response(),
            Error::AccessTokenHeaderNotFound | Error::UnAssociatedAccessToken => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, Json(ErrorResponse::from(self))).into_response()
            }
            Error::AccessTokenValidation(err) => err.into_response(),
            Error::DataAccess(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
