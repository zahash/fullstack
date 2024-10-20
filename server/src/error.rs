use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("{0}")]
    Public(#[from] PublicError),

    #[error("{0:?}")]
    Internal(InternalError),
}

#[derive(thiserror::Error, Debug)]
pub enum PublicError {
    #[error("{0}")]
    Auth(#[from] AuthError),
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

    #[error("multiple credentials provided {0:?}")]
    MultipleCredentialsProvided(Vec<&'static str>),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SessionError {
    #[error("invalid session")]
    InvalidSessionToken,

    #[error("malformed session token")]
    MalformedSessionToken,

    #[error("`session_id` cookie not found")]
    SessionCookieNotFound,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AccessTokenError {
    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenNotFound,

    #[error("invalid access token")]
    InvalidAccessToken,

    #[error("invalid access token format. must be in the form 'Token <your-access-token>'")]
    InvalidAccessTokenFormat,

    #[error("malformed access token")]
    MalformedAccessToken,
}

#[derive(thiserror::Error, Debug)]
#[error("{0:?}")]
pub struct InternalError(#[from] pub anyhow::Error);

#[derive(PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
    Security,
}

impl HandlerError {
    pub fn severity(&self) -> Severity {
        match self {
            HandlerError::Public(e) => e.severity(),
            HandlerError::Internal(e) => e.severity(),
        }
    }
}

impl PublicError {
    pub fn severity(&self) -> Severity {
        match self {
            PublicError::Auth(e) => e.severity(),
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

impl SessionError {
    pub fn severity(&self) -> Severity {
        match self {
            SessionError::InvalidSessionToken => Severity::Medium,
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
            AccessTokenError::InvalidAccessToken => Severity::Medium,
            AccessTokenError::InvalidAccessTokenFormat => Severity::Medium,
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
        match self.severity() {
            Severity::Low | Severity::Medium => tracing::info!("{:?}", self),
            Severity::High => tracing::warn!("{:?}", self),
            Severity::Critical => tracing::error!("{:?}", self),
            Severity::Security => tracing::error!("!SECURITY! {:?}", self),
        };

        match self {
            HandlerError::Public(e) => {
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
                };

                let now = OffsetDateTime::now_utc()
                    .format(&Iso8601::DATE_TIME_OFFSET)
                    .inspect_err(|e| {
                        tracing::warn!("unable to format OffsetDateTime::now_utc() :: {:?}", e)
                    })
                    .ok();

                let message = match e.severity() {
                    Severity::Security => "Security incident detected! This will be reported immediately!",
                    _ => "Please check the response headers for `x-request-id`, include the datetime and raise a support ticket.",
                };

                (
                    status_code,
                    Json(json!({
                        "error": e.to_string(),
                        "message": message,
                        "datetime": now,
                    })),
                )
                    .into_response()
            }
            HandlerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl From<AuthError> for HandlerError {
    fn from(err: AuthError) -> Self {
        HandlerError::Public(PublicError::Auth(err))
    }
}

impl From<SessionError> for PublicError {
    fn from(err: SessionError) -> Self {
        PublicError::Auth(AuthError::Session(err))
    }
}

impl From<SessionError> for HandlerError {
    fn from(err: SessionError) -> Self {
        HandlerError::Public(PublicError::from(err))
    }
}

impl From<AccessTokenError> for HandlerError {
    fn from(err: AccessTokenError) -> Self {
        HandlerError::Public(PublicError::Auth(AuthError::AccessToken(err)))
    }
}

impl From<anyhow::Error> for HandlerError {
    fn from(err: anyhow::Error) -> Self {
        HandlerError::Internal(InternalError(err))
    }
}

impl From<InternalError> for HandlerError {
    fn from(err: InternalError) -> Self {
        HandlerError::Internal(err)
    }
}
