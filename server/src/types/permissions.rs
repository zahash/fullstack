use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use time::OffsetDateTime;

use crate::{
    AppState,
    error::{Context, InternalError, error},
    types::{AccessToken, SessionId, UserId},
};

use super::{
    access_token::{AccessTokenExtractionError, AccessTokenValiationError},
    session_id::{SessionIdExtractionError, SessionIdValidationError},
};

// include username and access token name
pub enum Principal {
    UserId(UserId),
    AccessToken {
        access_token: AccessToken,
        owner: UserId,
    },
}

pub struct Permissions {
    permissions: Vec<String>,
    principal: Principal,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    AccessTokenExtraction(#[from] AccessTokenExtractionError),

    #[error("{0}")]
    SessionIdExtraction(#[from] SessionIdExtractionError),

    #[error("{0}")]
    SessionIdValidation(#[from] SessionIdValidationError),

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValiationError),

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
        let session_id = SessionId::try_from(parts as &Parts);
        let access_token = AccessToken::try_from(parts as &Parts);

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
                let session_id_hash = session_id.hash();

                let record = sqlx::query!(
                    "SELECT user_id, expires_at FROM sessions
                     WHERE session_id_hash = ?",
                    session_id_hash
                )
                .fetch_optional(&state.pool)
                .await
                .context("Auth :: select SessionId")?
                .ok_or(Error::SessionIdValidation(
                    SessionIdValidationError::UnAssociatedSessionId,
                ))?;

                if OffsetDateTime::now_utc() > record.expires_at {
                    return Err(Error::SessionIdValidation(
                        SessionIdValidationError::SessionExpired,
                    ));
                }

                let user_id = UserId::from(record.user_id);

                let permissions = sqlx::query!(
                    "SELECT p.permission from permissions p
                     INNER JOIN user_permissions up ON up.permission_id = p.id
                     WHERE up.user_id = ?",
                    user_id
                )
                .fetch_all(&state.pool)
                .await
                .context("UserId -> Permissions")?
                .into_iter()
                .map(|record| record.permission)
                .collect::<Vec<String>>();

                Ok(Self {
                    permissions,
                    principal: Principal::UserId(user_id),
                })
            }
            (_, Ok(access_token)) => {
                let access_token_hash = access_token.hash();

                let record = sqlx::query!(
                    "SELECT user_id, expires_at FROM access_tokens
                     WHERE access_token_hash = ?",
                    access_token_hash
                )
                .fetch_optional(&state.pool)
                .await
                .context("Auth :: select AccessToken")?
                .ok_or(Error::AccessTokenValidation(
                    AccessTokenValiationError::UnAssociatedAccessToken,
                ))?;

                if OffsetDateTime::now_utc() > record.expires_at {
                    return Err(Error::AccessTokenValidation(
                        AccessTokenValiationError::AccessTokenExpired,
                    ));
                }

                let user_id = UserId::from(record.user_id);

                let permissions = sqlx::query!(
                    "SELECT p.permission from permissions p
                     INNER JOIN access_token_permissions atp ON atp.permission_id = p.id
                     INNER JOIN access_tokens at ON atp.access_token_id = at.id
                     WHERE at.access_token_hash = ?",
                    access_token_hash
                )
                .fetch_all(&state.pool)
                .await
                .context("AccessToken -> Permissions")?
                .into_iter()
                .map(|record| record.permission)
                .collect::<Vec<String>>();

                Ok(Self {
                    permissions,
                    principal: Principal::AccessToken {
                        access_token,
                        owner: user_id,
                    },
                })
            }
            _ => Err(Error::NoCredentialsProvided),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("insufficient permissions")]
pub struct InsufficientPermissions;

impl Permissions {
    pub fn contains(&self, permission: &str) -> bool {
        self.permissions.iter().any(|s| s == permission)
    }

    pub fn require(&self, permission: &str) -> Result<(), InsufficientPermissions> {
        match self.contains(permission) {
            true => Ok(()),
            false => Err(InsufficientPermissions),
        }
    }

    pub fn user_id(&self) -> &UserId {
        match &self.principal {
            Principal::UserId(user_id) | Principal::AccessToken { owner: user_id, .. } => user_id,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::AccessTokenExtraction(err) => err.into_response(),
            Error::SessionIdExtraction(err) => err.into_response(),
            Error::SessionIdValidation(err) => err.into_response(),
            Error::AccessTokenValidation(err) => err.into_response(),
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

impl IntoResponse for InsufficientPermissions {
    fn into_response(self) -> axum::response::Response {
        tracing::info!("{:?}", self);
        (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
    }
}
