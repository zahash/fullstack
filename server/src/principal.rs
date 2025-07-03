use auth::{
    AccessToken, AccessTokenAuthorizationExtractionError, AccessTokenInfo,
    AccessTokenValidationError, Base64DecodeError, Basic, BasicAuthorizationExtractionError,
    Credentials, Permissions, SessionId, SessionInfo, SessionValidationError, UserInfo, Verified,
};
use boxer::{Boxer, Context};
use data_access::DataAccess;
use http::HeaderMap;

use crate::AppState;

pub enum Principal {
    Session(Verified<SessionInfo>),
    AccessToken(Verified<AccessTokenInfo>),
    Basic(Verified<UserInfo>),
}

#[derive(thiserror::Error, Debug)]
pub enum PrincipalError {
    #[error("{0}")]
    AccessTokenAuthorizationExtractionError(#[from] AccessTokenAuthorizationExtractionError),

    #[error("{0}")]
    BasicAuthorizationExtractionError(#[from] BasicAuthorizationExtractionError),

    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValidationError),

    #[error("{0}")]
    Base64Decode(#[from] Base64DecodeError),

    #[error("session id not associated with any user")]
    UnAssociatedSessionId,

    #[error("{0}")]
    SessionIdValidation(#[from] SessionValidationError),

    #[error("user with username {0} not found")]
    UsernameNotFound(String),

    #[error("invalid basic credentials")]
    InvalidBasicCredentials,

    #[error("no credentials provided")]
    NoCredentialsProvided,

    #[error("{0:?}")]
    Internal(#[from] Boxer),
}

impl Principal {
    pub fn user_id(&self) -> i64 {
        match self {
            Principal::Session(info) => info.user_id,
            Principal::AccessToken(info) => info.user_id,
            Principal::Basic(info) => info.user_id,
        }
    }

    pub async fn permissions(&self, data_access: &DataAccess) -> Result<Permissions, sqlx::Error> {
        match self {
            Principal::Session(info) => info.permissions(data_access).await,
            Principal::AccessToken(info) => info.permissions(data_access).await,
            Principal::Basic(info) => info.permissions(data_access).await,
        }
    }

    pub async fn from(
        headers: &HeaderMap,
        data_access: &DataAccess,
    ) -> Result<Self, PrincipalError> {
        if let Some(access_token) = AccessToken::try_from_headers(headers)? {
            let info = access_token
                .info(data_access)
                .await
                .context("AccessToken -> AccessTokenInfo")?
                .ok_or(PrincipalError::UnAssociatedAccessToken)?;
            let validated_info = info.verify()?;
            return Ok(Principal::AccessToken(validated_info));
        }

        if let Some(Basic { username, password }) = Basic::try_from_headers(headers)? {
            let user_info = UserInfo::from_username(&username, data_access)
                .await
                .context("username -> UserInfo")?
                .ok_or(PrincipalError::UsernameNotFound(username))?;
            let validated_info = user_info
                .verify_password(&password)
                .context("verify password hash")?
                .ok_or(PrincipalError::InvalidBasicCredentials)?;
            return Ok(Principal::Basic(validated_info));
        }

        if let Some(session_id) = SessionId::try_from_headers(headers)? {
            let info = session_id
                .info(data_access)
                .await
                .context("SessionId -> SessionInfo")?
                .ok_or(PrincipalError::UnAssociatedSessionId)?;
            let validated_info = info.validate()?;
            return Ok(Principal::Session(validated_info));
        }

        Err(PrincipalError::NoCredentialsProvided)
    }
}

impl axum::extract::FromRequestParts<AppState> for Principal {
    type Rejection = PrincipalError;

    async fn from_request_parts(
        axum::http::request::Parts { headers, .. }: &mut axum::http::request::Parts,
        AppState { data_access, .. }: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Principal::from(headers, data_access).await
    }
}

impl axum::response::IntoResponse for PrincipalError {
    fn into_response(self) -> axum::response::Response {
        match self {
            PrincipalError::UnAssociatedAccessToken
            | PrincipalError::UnAssociatedSessionId
            | PrincipalError::InvalidBasicCredentials
            | PrincipalError::NoCredentialsProvided
            | PrincipalError::UsernameNotFound(_) => {
                tracing::info!("{:?}", self);
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(extra::json_error_response(self)),
                )
                    .into_response()
            }
            PrincipalError::AccessTokenAuthorizationExtractionError(err) => err.into_response(),
            PrincipalError::BasicAuthorizationExtractionError(err) => err.into_response(),
            PrincipalError::AccessTokenValidation(err) => err.into_response(),
            PrincipalError::SessionIdValidation(err) => err.into_response(),
            PrincipalError::Base64Decode(err) => err.into_response(),
            PrincipalError::Internal(err) => {
                tracing::error!("{:?}", err);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
