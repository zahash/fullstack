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
    pub request_id: RequestId,
    pub kind: HandlerErrorKind,
}

#[derive(thiserror::Error, Debug)]
pub enum HandlerErrorKind {
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
            HandlerErrorKind::Public(e) => {
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

                let now = OffsetDateTime::now_utc()
                    .format(&Iso8601::DATE_TIME_OFFSET)
                    .inspect_err(|e| {
                        tracing::warn!("unable to format OffsetDateTime::now_utc() :: {:?}", e)
                    })
                    .ok();

                (
                    status_code,
                    Json(json!({
                        "message": e.to_string(),
                        "datetime": now,
                        "request_id": self.request_id
                    })),
                )
                    .into_response()
            }
            HandlerErrorKind::Internal(e) => {
                tracing::error!("{:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl From<AuthError> for HandlerErrorKind {
    fn from(err: AuthError) -> Self {
        HandlerErrorKind::Public(PublicError::Auth(err))
    }
}

impl From<CookieError> for HandlerErrorKind {
    fn from(err: CookieError) -> Self {
        HandlerErrorKind::Public(PublicError::Cookie(err))
    }
}
