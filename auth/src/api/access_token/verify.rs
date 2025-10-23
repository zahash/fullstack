use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use contextual::Context;
use extra::ErrorResponse;

use crate::{
    AppState,
    core::{
        AccessToken, AccessTokenAuthorizationExtractionError, AccessTokenValidationError,
        Credentials,
    },
};

pub const PATH: &str = "/access-token/verify";

pub fn method_router() -> MethodRouter<AppState> {
    get(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    operation_id = PATH,
    responses(
        (status = 200, description = "Access token is valid"),
        (status = 401, description = "Invalid or missing access token", body = ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("Authorization" = String, Header, description = "Access token in the form 'Token <your-access-token>'")
    ),
    tag = "access_token"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(skip_all, ret))]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, Error> {
    let access_token =
        AccessToken::try_from_headers(&headers)?.ok_or_else(|| Error::AccessTokenHeaderNotFound)?;

    let info = access_token
        .info(&pool)
        .await
        .context("AccessToken -> AccessTokenInfo")?
        .ok_or(Error::UnAssociatedAccessToken)?;

    #[cfg(feature = "tracing")]
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
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::AccessTokenAuthorizationExtractionError(err) => err.kind(),
            Error::AccessTokenHeaderNotFound => "auth.access-token.authorization-header.not-found",
            Error::UnAssociatedAccessToken => "auth.access-token.unassociated",
            Error::AccessTokenValidation(err) => err.kind(),
            Error::Sqlx(_) => "auth.access-token.sqlx",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::AccessTokenAuthorizationExtractionError(err) => err.into_response(),
            Error::AccessTokenHeaderNotFound | Error::UnAssociatedAccessToken => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::UNAUTHORIZED, Json(ErrorResponse::from(self))).into_response()
            }
            Error::AccessTokenValidation(err) => err.into_response(),
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
