use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_macros::debug_handler;

use server_core::{
    AccessTokenValiationError, AppState, AuthorizationHeader, AuthorizationHeaderError, Context,
    InternalError, error,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    AuthorizationHeader(#[from] AuthorizationHeaderError),

    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenHeaderNotFound,

    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

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
    let Some(AuthorizationHeader::AccessToken(access_token)) =
        AuthorizationHeader::try_from_headers(&headers)?
    else {
        return Err(Error::AccessTokenHeaderNotFound);
    };

    let info = access_token
        .info(&pool)
        .await
        .context("AccessToken -> AccessTokenInfo")?
        .ok_or(Error::UnAssociatedAccessToken)?;

    info.validate()?;

    Ok(StatusCode::OK)
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::AuthorizationHeader(err) => err.into_response(),
            Error::AccessTokenHeaderNotFound | Error::UnAssociatedAccessToken => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            Error::AccessTokenValidation(err) => err.into_response(),
            Error::Internal(err) => err.into_response(),
        }
    }
}
