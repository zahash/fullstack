use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};

use crate::{
    AccessTokenInfo, AccessTokenValiationError, AppState, AuthorizationHeader,
    AuthorizationHeaderError, Base64DecodeError, Context, InternalError, SessionId, SessionInfo,
    SessionValidationError, UserId, UserInfo, error,
};

pub enum Principal {
    Session(SessionInfo),
    AccessToken(AccessTokenInfo),
    Basic(UserInfo),
}

pub struct Permissions {
    pub permissions: Vec<String>,
    pub principal: Principal,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    AuthorizationHeader(#[from] AuthorizationHeaderError),

    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValiationError),

    #[error("{0}")]
    Base64Decode(#[from] Base64DecodeError),

    #[error("session id not associated with any user")]
    UnAssociatedSessionId,

    #[error("{0}")]
    SessionIdValidation(#[from] SessionValidationError),

    #[error("invalid basic credentials")]
    InvalidBasicCredentials,

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
        if let Some(authorization_header) = AuthorizationHeader::try_from_headers(&parts.headers)? {
            match authorization_header {
                AuthorizationHeader::AccessToken(access_token) => {
                    let info = access_token
                        .info(&state.pool)
                        .await
                        .context("AccessToken -> AccessTokenInfo")?
                        .ok_or(Error::UnAssociatedAccessToken)?;
                    let validated_info = info.validate()?;
                    let permissions = validated_info
                        .permissions(&state.pool)
                        .await
                        .context("Valid<AccessTokenInfo> -> Permissions")?;
                    return Ok(Permissions {
                        permissions,
                        principal: Principal::AccessToken(validated_info.inner()),
                    });
                }
                AuthorizationHeader::Basic { username, password } => {
                    let user_info = UserInfo::from_username(&username, &state.pool)
                        .await
                        .context("username -> UserInfo")?
                        .ok_or(Error::InvalidBasicCredentials)?;
                    let validated_info = user_info
                        .verify_password(&password)
                        .context("verify password hash")?
                        .ok_or(Error::InvalidBasicCredentials)?;
                    let permissions = validated_info
                        .permissions(&state.pool)
                        .await
                        .context("Valid<UserInfo> -> Permissions")?;
                    return Ok(Permissions {
                        permissions,
                        principal: Principal::Basic(validated_info.inner()),
                    });
                }
            }
        }

        if let Some(session_id) = SessionId::try_from_headers(&parts.headers)? {
            let info = session_id
                .info(&state.pool)
                .await
                .context("SessionId -> SessionInfo")?
                .ok_or(Error::UnAssociatedSessionId)?;
            let validated_info = info.validate()?;
            let permissions = validated_info
                .permissions(&state.pool)
                .await
                .context("Valid<SessionInfo> -> Permissions")?;
            return Ok(Permissions {
                permissions,
                principal: Principal::Session(validated_info.inner()),
            });
        }

        Err(Error::NoCredentialsProvided)
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
            | Principal::AccessToken(AccessTokenInfo { user_id, .. })
            | Principal::Basic(UserInfo { user_id, .. }) => user_id,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::AuthorizationHeader(err) => err.into_response(),
            Error::UnAssociatedAccessToken
            | Error::UnAssociatedSessionId
            | Error::InvalidBasicCredentials
            | Error::Base64Decode(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            Error::SessionIdValidation(err) => err.into_response(),
            Error::AccessTokenValidation(err) => err.into_response(),
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
        (StatusCode::FORBIDDEN, error(&self.to_string())).into_response()
    }
}
