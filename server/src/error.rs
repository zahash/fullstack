use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

use crate::{access_token::AccessToken, request_id::RequestId, session_id::SessionId};

#[derive(Debug)]
pub struct HandlerError {
    pub request_id: Option<RequestId>,
    pub kind: HandlerErrorKind,
}

#[derive(thiserror::Error, Debug)]
pub enum HandlerErrorKind {
    #[error("{0}")]
    Public(#[from] PublicError),

    #[error("{0:?}")]
    Internal(InternalError),
}

#[derive(thiserror::Error, Debug)]
pub enum PublicError {
    #[error("{0}")]
    Auth(#[from] AuthError),

    #[error("{0}")]
    Cookie(#[from] CookieError),
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("user '{0}' not found")]
    UserNotFound(String),

    #[error("password mismatch")]
    PasswordMismatch,

    #[error("username '{0}' is already taken")]
    UsernameTaken(String),

    #[error("invalid username '{username}'. reason: {reason}")]
    InvalidUsername { username: String, reason: String },

    #[error("{0}")]
    Session(#[from] SessionError),

    #[error("{0}")]
    AccessToken(AccessTokenError),

    #[error("no credentials provided")]
    NoCredentialsProvided,

    #[error("multiple credentials provided")]
    MultipleCredentialsProvided {
        session_id: SessionId,
        access_token: AccessToken,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum CookieError {
    #[error("cookie not found: '{0}'")]
    CookieNotFound(&'static str),
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("invalid session")]
    InvalidSessionToken,

    #[error("malformed session token")]
    MalformedSessionToken,

    #[error("`session_id` cookie not found")]
    SessionCookieNotFound,
}

#[derive(thiserror::Error, Debug)]
pub enum AccessTokenError {
    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenNotFound,

    #[error("invalid access token")]
    InvalidAccessToken,

    #[error("malformed access token")]
    MalformedAccessToken,
}

#[derive(thiserror::Error, Debug)]
#[error("{0:?}")]
pub struct InternalError(#[from] pub anyhow::Error);

pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
    Security,
}

impl HandlerErrorKind {
    pub fn severity(&self) -> Severity {
        match self {
            HandlerErrorKind::Public(e) => e.severity(),
            HandlerErrorKind::Internal(e) => e.severity(),
        }
    }
}

impl PublicError {
    pub fn severity(&self) -> Severity {
        match self {
            PublicError::Auth(e) => e.severity(),
            PublicError::Cookie(e) => e.severity(),
        }
    }
}

impl AuthError {
    pub fn severity(&self) -> Severity {
        match self {
            AuthError::UserNotFound(_) => Severity::Low,
            AuthError::PasswordMismatch => Severity::High,
            AuthError::UsernameTaken(_) => Severity::Low,
            AuthError::InvalidUsername { .. } => Severity::Low,
            AuthError::Session(e) => e.severity(),
            AuthError::AccessToken(e) => e.severity(),
            AuthError::NoCredentialsProvided => Severity::Low,
            AuthError::MultipleCredentialsProvided { .. } => Severity::High,
        }
    }
}

impl CookieError {
    pub fn severity(&self) -> Severity {
        match self {
            CookieError::CookieNotFound(_) => Severity::Low,
        }
    }
}

impl SessionError {
    pub fn severity(&self) -> Severity {
        match self {
            SessionError::InvalidSessionToken => Severity::Low,
            SessionError::MalformedSessionToken => Severity::Security,
            SessionError::SessionCookieNotFound => Severity::Low,
        }
    }
}

impl AccessTokenError {
    pub fn severity(&self) -> Severity {
        match self {
            AccessTokenError::AccessTokenNotFound => Severity::Low,
            AccessTokenError::MalformedAccessToken => Severity::Security,
            AccessTokenError::InvalidAccessToken => Severity::Low,
        }
    }
}

impl InternalError {
    pub fn severity(&self) -> Severity {
        Severity::Critical
    }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        match self.kind.severity() {
            Severity::Low | Severity::Medium => tracing::info!("{:?}", self.kind),
            Severity::High => tracing::warn!("{:?}", self.kind),
            Severity::Critical => tracing::error!("{:?}", self.kind),
            Severity::Security => tracing::error!(":SECURITY: {:?}", self.kind),
        };

        match self.kind {
            HandlerErrorKind::Public(e) => {
                let status_code = match &e {
                    PublicError::Auth(e) => match e {
                        AuthError::UserNotFound(_) => StatusCode::NOT_FOUND,
                        AuthError::PasswordMismatch
                        | AuthError::Session(_)
                        | AuthError::AccessToken(_)
                        | AuthError::NoCredentialsProvided => StatusCode::UNAUTHORIZED,
                        AuthError::UsernameTaken(_) => StatusCode::CONFLICT,
                        AuthError::InvalidUsername {
                            username: _,
                            reason: _,
                        } => StatusCode::BAD_REQUEST,
                        AuthError::MultipleCredentialsProvided { .. } => StatusCode::FORBIDDEN,
                    },
                    PublicError::Cookie(e) => match e {
                        CookieError::CookieNotFound(_) => StatusCode::BAD_REQUEST,
                    },
                };

                let now = OffsetDateTime::now_utc()
                    .format(&Iso8601::DATE_TIME_OFFSET)
                    .inspect_err(|e| {
                        tracing::warn!("unable to format OffsetDateTime::now_utc() :: {:?}", e)
                    })
                    .ok();

                let message = match e.severity() {
                    Severity::Low | Severity::Medium | Severity::High | Severity::Critical => {
                        e.to_string()
                    }
                    Severity::Security => format!("{}. this incident will be reported!", e),
                };

                (
                    status_code,
                    Json(json!({
                        "message": message,
                        "datetime": now,
                        "request_id": self.request_id
                    })),
                )
                    .into_response()
            }
            HandlerErrorKind::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl From<AuthError> for HandlerErrorKind {
    fn from(err: AuthError) -> Self {
        HandlerErrorKind::Public(PublicError::Auth(err))
    }
}

impl From<SessionError> for PublicError {
    fn from(err: SessionError) -> Self {
        PublicError::Auth(AuthError::Session(err))
    }
}

impl From<SessionError> for HandlerErrorKind {
    fn from(err: SessionError) -> Self {
        HandlerErrorKind::Public(PublicError::from(err))
    }
}

impl From<AccessTokenError> for HandlerErrorKind {
    fn from(err: AccessTokenError) -> Self {
        HandlerErrorKind::Public(PublicError::Auth(AuthError::AccessToken(err)))
    }
}

impl From<anyhow::Error> for HandlerErrorKind {
    fn from(err: anyhow::Error) -> Self {
        HandlerErrorKind::Internal(InternalError(err))
    }
}

impl From<InternalError> for HandlerErrorKind {
    fn from(err: InternalError) -> Self {
        HandlerErrorKind::Internal(err)
    }
}
