use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};

use crate::{
    AccessTokenInfo, AccessTokenValiationError, AppState, AuthorizationHeader,
    AuthorizationHeaderError, Base64DecodeError, Context, DataAccess, InternalError, SessionId,
    SessionInfo, SessionValidationError, UserInfo, Valid, error,
};

pub enum Principal {
    Session(Valid<SessionInfo>),
    AccessToken(Valid<AccessTokenInfo>),
    Basic(Valid<UserInfo>),
}

pub struct Permissions {
    permissions: Vec<Permission>,
}

#[derive(Clone)]
pub struct Permission {
    pub id: i64,
    pub permission: String,
    pub description: Option<String>,
}

#[derive(thiserror::Error, Debug)]
#[error("insufficient permissions")]
pub struct InsufficientPermissionsError;

#[derive(thiserror::Error, Debug)]
pub enum PrincipalError {
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
            Principal::Session(info) => info
                .permissions(data_access)
                .await
                .map(|permissions| Permissions { permissions }),
            Principal::AccessToken(info) => info
                .permissions(data_access)
                .await
                .map(|permissions| Permissions { permissions }),
            Principal::Basic(info) => info
                .permissions(data_access)
                .await
                .map(|permissions| Permissions { permissions }),
        }
    }
}

impl Permissions {
    pub fn contains(&self, permission: &str) -> bool {
        self.permissions
            .iter()
            .map(|p| &p.permission)
            .any(|s| s == permission)
    }

    pub fn require(&self, permission: &str) -> Result<(), InsufficientPermissionsError> {
        match self.contains(permission) {
            true => Ok(()),
            false => Err(InsufficientPermissionsError),
        }
    }
}

impl FromRequestParts<AppState> for Principal {
    type Rejection = PrincipalError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(authorization_header) = AuthorizationHeader::try_from_headers(&parts.headers)? {
            match authorization_header {
                AuthorizationHeader::AccessToken(access_token) => {
                    let info = access_token
                        .info(&state.data_access)
                        .await
                        .context("AccessToken -> AccessTokenInfo")?
                        .ok_or(PrincipalError::UnAssociatedAccessToken)?;
                    let validated_info = info.validate()?;
                    return Ok(Principal::AccessToken(validated_info));
                }
                AuthorizationHeader::Basic { username, password } => {
                    let user_info = UserInfo::from_username(&username, &state.data_access)
                        .await
                        .context("username -> UserInfo")?
                        .ok_or(PrincipalError::InvalidBasicCredentials)?;
                    let validated_info = user_info
                        .verify_password(&password)
                        .context("verify password hash")?
                        .ok_or(PrincipalError::InvalidBasicCredentials)?;
                    return Ok(Principal::Basic(validated_info));
                }
            }
        }

        if let Some(session_id) = SessionId::try_from_headers(&parts.headers)? {
            let info = session_id
                .info(&state.data_access)
                .await
                .context("SessionId -> SessionInfo")?
                .ok_or(PrincipalError::UnAssociatedSessionId)?;
            let validated_info = info.validate()?;
            return Ok(Principal::Session(validated_info));
        }

        Err(PrincipalError::NoCredentialsProvided)
    }
}

impl IntoResponse for PrincipalError {
    fn into_response(self) -> axum::response::Response {
        match self {
            PrincipalError::AuthorizationHeader(err) => err.into_response(),
            PrincipalError::UnAssociatedAccessToken
            | PrincipalError::UnAssociatedSessionId
            | PrincipalError::InvalidBasicCredentials
            | PrincipalError::Base64Decode(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            PrincipalError::SessionIdValidation(err) => err.into_response(),
            PrincipalError::AccessTokenValidation(err) => err.into_response(),
            PrincipalError::NoCredentialsProvided => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            PrincipalError::Internal(err) => err.into_response(),
        }
    }
}

impl IntoResponse for InsufficientPermissionsError {
    fn into_response(self) -> axum::response::Response {
        tracing::info!("{:?}", self);
        (StatusCode::FORBIDDEN, error(&self.to_string())).into_response()
    }
}
