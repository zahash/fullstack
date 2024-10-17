use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

use crate::request_id::RequestId;

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

    #[error("user IDs from session id and access token do not match")]
    UserIdMismatch,

    #[error("no credentials provided")]
    NoCredentialsProvided,
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
        use HandlerErrorKind::*;

        match self {
            Public(e) => e.severity(),
            Internal(e) => e.severity(),
        }
    }
}

impl PublicError {
    pub fn severity(&self) -> Severity {
        use PublicError::*;

        match self {
            Auth(e) => e.severity(),
            Cookie(e) => e.severity(),
        }
    }
}

impl AuthError {
    pub fn severity(&self) -> Severity {
        use AuthError::*;
        use Severity::*;

        match self {
            UserNotFound(_) => Low,
            PasswordMismatch => High,
            UsernameTaken(_) => Low,
            InvalidUsername { .. } => Low,
            Session(e) => e.severity(),
            AccessToken(e) => e.severity(),
            UserIdMismatch => Security,
            NoCredentialsProvided => Low,
        }
    }
}

impl CookieError {
    pub fn severity(&self) -> Severity {
        use CookieError::*;
        use Severity::*;

        match self {
            CookieNotFound(_) => Low,
        }
    }
}

impl SessionError {
    pub fn severity(&self) -> Severity {
        use SessionError::*;
        use Severity::*;

        match self {
            InvalidSessionToken => Low,
            MalformedSessionToken => Security,
            SessionCookieNotFound => Low,
        }
    }
}

impl AccessTokenError {
    pub fn severity(&self) -> Severity {
        use AccessTokenError::*;
        use Severity::*;

        match self {
            AccessTokenNotFound => Low,
            MalformedAccessToken => Security,
            InvalidAccessToken => Low,
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
        use HandlerErrorKind::*;
        use Severity::*;

        match self.kind.severity() {
            Low | Medium => tracing::info!("{:?}", self.kind),
            High => tracing::warn!("{:?}", self.kind),
            Critical => tracing::error!("{:?}", self.kind),
            Security => tracing::error!(":SECURITY: {:?}", self.kind),
        };

        match self.kind {
            Public(e) => {
                let status_code = match &e {
                    PublicError::Auth(e) => {
                        use AuthError::*;
                        match e {
                            UserNotFound(_) => StatusCode::NOT_FOUND,
                            PasswordMismatch
                            | Session(_)
                            | AccessToken(_)
                            | NoCredentialsProvided => StatusCode::UNAUTHORIZED,
                            UsernameTaken(_) => StatusCode::CONFLICT,
                            InvalidUsername {
                                username: _,
                                reason: _,
                            } => StatusCode::BAD_REQUEST,
                            UserIdMismatch => StatusCode::FORBIDDEN,
                        }
                    }
                    PublicError::Cookie(e) => {
                        use CookieError::*;
                        match e {
                            CookieNotFound(_) => StatusCode::BAD_REQUEST,
                        }
                    }
                };

                let now = OffsetDateTime::now_utc()
                    .format(&Iso8601::DATE_TIME_OFFSET)
                    .inspect_err(|e| {
                        tracing::warn!("unable to format OffsetDateTime::now_utc() :: {:?}", e)
                    })
                    .ok();

                let message = match e.severity() {
                    Low | Medium | High | Critical => e.to_string(),
                    Security => format!("{}. this incident will be reported!", e),
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
            Internal(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
