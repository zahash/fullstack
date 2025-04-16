use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};

use crate::{
    AppState,
    error::{Context, InternalError, error},
    types::{
        AccessToken, AccessTokenInfo, AccessTokenInfoError, SessionId, SessionInfo,
        SessionInfoError, UserId,
    },
};

use super::{
    access_token::{AccessTokenExtractionError, AccessTokenValiationError},
    session_id::{SessionIdExtractionError, SessionValidationError},
};

// TODO: include Basic auth in this
pub enum Principal {
    Session(SessionInfo),
    AccessToken(AccessTokenInfo),
}

pub struct Permissions {
    pub permissions: Vec<String>,
    pub principal: Principal,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    SessionIdExtraction(#[from] SessionIdExtractionError),

    #[error("{0}")]
    AccessTokenExtraction(#[from] AccessTokenExtractionError),

    #[error("{0}")]
    SessionIdValidation(#[from] SessionValidationError),

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValiationError),

    #[error("{0}")]
    SessionInfo(#[from] SessionInfoError),

    #[error("{0}")]
    AccessTokenInfo(#[from] AccessTokenInfoError),

    #[error("multiple credentials provided {0:?}")]
    MultipleCredentialsProvided(Vec<&'static str>),

    #[error("no credentials provided")]
    NoCredentialsProvided,

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

impl FromRequestParts<AppState> for Permissions {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session_id = SessionId::try_from(&parts.headers);
        let access_token = AccessToken::try_from(&parts.headers);

        match (session_id, access_token) {
            (Ok(_), Ok(_)) => Err(Error::MultipleCredentialsProvided(vec![
                "SessionId",
                "AccessToken",
            ])),
            (Err(err), _) if err != SessionIdExtractionError::SessionCookieNotFound => {
                Err(err.into())
            }
            (_, Err(err)) if err != AccessTokenExtractionError::AccessTokenHeaderNotFound => {
                Err(err.into())
            }
            (Ok(session_id), _) => {
                let info = session_id.info(&state.pool).await?;
                let validated_info = info.validate()?;
                let permissions = validated_info
                    .permissions(&state.pool)
                    .await
                    .context("Valid<SessionInfo> -> Permissions")?;
                Ok(permissions)
            }
            (_, Ok(access_token)) => {
                let info = access_token.info(&state.pool).await?;
                let validated_info = info.validate()?;
                let permissions = validated_info
                    .permissions(&state.pool)
                    .await
                    .context("Valid<AccessTokenInfo> -> Permissions")?;
                Ok(permissions)
            }
            _ => Err(Error::NoCredentialsProvided),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("insufficient permissions")]
pub struct InsufficientPermissionsError;

impl Permissions {
    pub fn contains(&self, permission: &str) -> bool {
        self.permissions.iter().any(|s| s == permission)
    }

    pub fn require(&self, permission: &str) -> Result<(), InsufficientPermissionsError> {
        match self.contains(permission) {
            true => Ok(()),
            false => Err(InsufficientPermissionsError),
        }
    }

    pub fn user_id(&self) -> &UserId {
        match &self.principal {
            Principal::Session(SessionInfo { user_id, .. })
            | Principal::AccessToken(AccessTokenInfo { user_id, .. }) => user_id,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::SessionIdExtraction(err) => err.into_response(),
            Error::AccessTokenExtraction(err) => err.into_response(),
            Error::SessionIdValidation(err) => err.into_response(),
            Error::AccessTokenValidation(err) => err.into_response(),
            Error::SessionInfo(err) => err.into_response(),
            Error::AccessTokenInfo(err) => err.into_response(),
            Error::MultipleCredentialsProvided(_) => {
                tracing::warn!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            Error::NoCredentialsProvided => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            Error::Internal(err) => err.into_response(),
        }
    }
}

impl IntoResponse for InsufficientPermissionsError {
    fn into_response(self) -> axum::response::Response {
        tracing::info!("{:?}", self);
        (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
    }
}
