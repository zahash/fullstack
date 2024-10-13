use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use time::OffsetDateTime;

use crate::types::RequestId;

#[derive(Debug)]
pub struct HandlerError {
    request_id: RequestId,
    kind: ErrorKind,
}

pub trait RequestIdCtx<T>
where
    Self: Sized,
{
    fn request_id(self, request_id: RequestId) -> Result<T, HandlerError>;
}

impl<T, E> RequestIdCtx<T> for Result<T, E>
where
    E: Into<ErrorKind>,
{
    fn request_id(self, request_id: RequestId) -> Result<T, HandlerError> {
        self.map_err(|e| HandlerError {
            request_id,
            kind: e.into(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("{0}")]
    Public(#[from] PublicError),

    #[error("{0:?}")]
    Internal(#[from] anyhow::Error),
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

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("username '{0}' is already taken")]
    UsernameTaken(String),

    #[error("invalid username '{username}'. reason: {reason}")]
    InvalidUsername { username: String, reason: String },

    #[error("invalid session")]
    InvalidSession,
}

#[derive(thiserror::Error, Debug)]
pub enum CookieError {
    #[error("cookie not found: '{0}'")]
    CookieNotFound(&'static str),
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        match self.kind {
            ErrorKind::Public(e) => {
                tracing::info!("{:?}", e);

                let status_code = match &e {
                    PublicError::Auth(e) => {
                        use AuthError::*;
                        match e {
                            UserNotFound(_) => StatusCode::NOT_FOUND,
                            InvalidCredentials | InvalidSession => StatusCode::UNAUTHORIZED,
                            UsernameTaken(_) => StatusCode::CONFLICT,
                            InvalidUsername {
                                username: _,
                                reason: _,
                            } => StatusCode::BAD_REQUEST,
                        }
                    }
                    PublicError::Cookie(e) => {
                        use CookieError::*;
                        match e {
                            CookieNotFound(_) => StatusCode::BAD_REQUEST,
                        }
                    }
                };

                (
                    status_code,
                    Json(json!(
                    {
                        "message": e.to_string(),
                        "datetime": OffsetDateTime::now_utc(),
                        "request_id": self.request_id
                    })),
                )
                    .into_response()
            }
            ErrorKind::Internal(e) => {
                tracing::error!("{:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl From<AuthError> for ErrorKind {
    fn from(err: AuthError) -> Self {
        ErrorKind::Public(PublicError::Auth(err))
    }
}

impl From<CookieError> for ErrorKind {
    fn from(err: CookieError) -> Self {
        ErrorKind::Public(PublicError::Cookie(err))
    }
}
