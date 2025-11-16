use std::fmt::Display;

use axum::{
    Json,
    extract::{FromRef, FromRequestParts},
    response::{IntoResponse, Response},
};
use contextual::Context;
use http::{HeaderMap, StatusCode, request::Parts};

use crate::core::{
    AccessToken, AccessTokenAuthorizationExtractionError, AccessTokenInfo,
    AccessTokenValidationError, Basic, BasicAuthorizationExtractionError, Credentials,
    InsufficientPermissionsError, Permission, SessionCookieExtractionError, SessionId, SessionInfo,
    SessionValidationError, UserInfo, Verified, permission::Authorizable,
};

pub enum Principal {
    Session(Verified<SessionInfo>),
    AccessToken(Verified<AccessTokenInfo>),
    Basic(Verified<UserInfo>),
}

#[derive(thiserror::Error, Debug)]
pub enum PrincipalError {
    #[error("{0}")]
    AccessTokenAuthorizationExtraction(#[from] AccessTokenAuthorizationExtractionError),

    #[error("{0}")]
    BasicAuthorizationExtraction(#[from] BasicAuthorizationExtractionError),

    #[error("{0}")]
    SessionCookieExtraction(#[from] SessionCookieExtractionError),

    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValidationError),

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

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),

    #[error("{0}")]
    Bcrypt(#[from] contextual::Error<bcrypt::BcryptError>),
}

impl Principal {
    pub fn user_id(&self) -> i64 {
        match self {
            Principal::Session(info) => info.user_id,
            Principal::AccessToken(info) => info.user_id,
            Principal::Basic(info) => info.user_id,
        }
    }

    pub async fn require_permission<E>(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        permission: &str,
    ) -> Result<(), E>
    where
        E: std::error::Error
            + From<InsufficientPermissionsError>
            + From<contextual::Error<sqlx::Error>>,
    {
        match self {
            Principal::Session(info) => info.require_permission(pool, permission).await,
            Principal::AccessToken(info) => info.require_permission(pool, permission).await,
            Principal::Basic(info) => info.require_permission(pool, permission).await,
        }
    }

    pub async fn has_permission(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        match self {
            Principal::Session(info) => info.has_permission(pool, permission).await,
            Principal::AccessToken(info) => info.has_permission(pool, permission).await,
            Principal::Basic(info) => info.has_permission(pool, permission).await,
        }
    }

    pub async fn permissions(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        match self {
            Principal::Session(info) => info.permissions(pool).await,
            Principal::AccessToken(info) => info.permissions(pool).await,
            Principal::Basic(info) => info.permissions(pool).await,
        }
    }

    pub async fn from(
        headers: &HeaderMap,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Self, PrincipalError> {
        if let Some(access_token) = AccessToken::try_from_headers(headers)? {
            let info = access_token
                .info(pool)
                .await
                .context("AccessToken -> AccessTokenInfo")?
                .ok_or(PrincipalError::UnAssociatedAccessToken)?;
            let validated_info = info.verify()?;
            return Ok(Principal::AccessToken(validated_info));
        }

        if let Some(Basic { username, password }) = Basic::try_from_headers(headers)? {
            let user_info = UserInfo::from_username(&username, pool)
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
                .info(pool)
                .await
                .context("SessionId -> SessionInfo")?
                .ok_or(PrincipalError::UnAssociatedSessionId)?;
            let validated_info = info.validate()?;
            return Ok(Principal::Session(validated_info));
        }

        Err(PrincipalError::NoCredentialsProvided)
    }
}

impl<S> FromRequestParts<S> for Principal
where
    S: Send + Sync,
    sqlx::Pool<sqlx::Sqlite>: FromRef<S>,
{
    type Rejection = PrincipalError;

    async fn from_request_parts(
        Parts { headers, .. }: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Principal::from(headers, &sqlx::Pool::<sqlx::Sqlite>::from_ref(state)).await
    }
}

impl extra::ErrorKind for PrincipalError {
    fn kind(&self) -> &'static str {
        match self {
            PrincipalError::UnAssociatedAccessToken => "auth.access-token.unassociated",
            PrincipalError::UnAssociatedSessionId => "auth.session.id.unassociated",
            PrincipalError::InvalidBasicCredentials => "auth.basic.invalid-credentials",
            PrincipalError::NoCredentialsProvided => "auth.no-credentials",
            PrincipalError::UsernameNotFound(_) => "auth.basic.username.not-found",
            PrincipalError::AccessTokenAuthorizationExtraction(err) => err.kind(),
            PrincipalError::BasicAuthorizationExtraction(err) => err.kind(),
            PrincipalError::SessionCookieExtraction(err) => err.kind(),
            PrincipalError::AccessTokenValidation(err) => err.kind(),
            PrincipalError::SessionIdValidation(err) => err.kind(),
            PrincipalError::Sqlx(_) => "auth.sqlx",
            PrincipalError::Bcrypt(_) => "auth.bcrypt",
        }
    }
}

impl IntoResponse for PrincipalError {
    fn into_response(self) -> Response {
        match self {
            PrincipalError::UnAssociatedAccessToken
            | PrincipalError::UnAssociatedSessionId
            | PrincipalError::InvalidBasicCredentials
            | PrincipalError::NoCredentialsProvided
            | PrincipalError::UsernameNotFound(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
            PrincipalError::AccessTokenAuthorizationExtraction(err) => err.into_response(),
            PrincipalError::BasicAuthorizationExtraction(err) => err.into_response(),
            PrincipalError::SessionCookieExtraction(err) => err.into_response(),
            PrincipalError::AccessTokenValidation(err) => err.into_response(),
            PrincipalError::SessionIdValidation(err) => err.into_response(),
            PrincipalError::Sqlx(_) | PrincipalError::Bcrypt(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl Display for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Principal::Session(session_info) => {
                write!(f, "Principal::Session::(user_id: {})", session_info.user_id)
            }
            Principal::AccessToken(access_token_info) => write!(
                f,
                "Principal::AccessToken::(user_id: {}, token_name: `{}`)",
                access_token_info.user_id, access_token_info.name
            ),
            Principal::Basic(user_info) => {
                write!(f, "Principal::Basic::(user_id: {})", user_info.user_id)
            }
        }
    }
}
