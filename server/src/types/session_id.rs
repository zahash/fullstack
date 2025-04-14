use std::ops::Deref;

use axum::{
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use time::OffsetDateTime;

use crate::{
    error::{error, security_error},
    token::Token,
};

#[derive(Debug)]
pub struct SessionId(Token<32>);

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SessionIdExtractionError {
    #[error("`session_id` cookie not found")]
    SessionCookieNotFound,

    #[error("malformed session token")]
    MalformedSessionToken,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionIdValidationError {
    #[error("session id not associated with any user")]
    UnAssociatedSessionId,

    #[error("session expired")]
    SessionExpired,
}

impl SessionId {
    pub fn new() -> Self {
        Self(Token::new())
    }

    pub fn into_cookie<'a>(self, expires_at: OffsetDateTime) -> Cookie<'a> {
        Cookie::build(("session_id", self.base64encoded()))
            .path("/")
            .same_site(SameSite::Strict)
            .expires(expires_at)
            .http_only(true)
            .secure(true)
            .build()
    }
}

pub trait SessionExt {
    fn remove_session_cookie(self) -> Self;
}

impl SessionExt for CookieJar {
    fn remove_session_cookie(self) -> Self {
        self.remove("session_id")
    }
}

impl Deref for SessionId {
    type Target = Token<32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&CookieJar> for SessionId {
    type Error = SessionIdExtractionError;

    fn try_from(jar: &CookieJar) -> Result<Self, Self::Error> {
        let value = jar
            .get("session_id")
            .ok_or(SessionIdExtractionError::SessionCookieNotFound)?
            .value();
        let token = Token::base64decode(value)
            .map_err(|_| SessionIdExtractionError::MalformedSessionToken)?;
        Ok(SessionId(token))
    }
}

impl TryFrom<&Parts> for SessionId {
    type Error = SessionIdExtractionError;

    fn try_from(parts: &Parts) -> Result<Self, Self::Error> {
        let jar = CookieJar::from_headers(&parts.headers);
        SessionId::try_from(&jar)
    }
}

// for building session cookie
// impl From<SessionId> for Cow<'_, str> {
//     fn from(value: SessionId) -> Self {
//         value.base64encoded().into()
//     }
// }

// impl<S: Send + Sync> FromRequestParts<S> for SessionId {
//     type Rejection = SessionIdExtractionError;

//     async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
//         SessionId::try_from(parts as &Parts)
//     }
// }

impl IntoResponse for SessionIdExtractionError {
    fn into_response(self) -> Response {
        match self {
            SessionIdExtractionError::SessionCookieNotFound => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            SessionIdExtractionError::MalformedSessionToken => {
                tracing::error!("!SECURITY! {:?}", self);
                (StatusCode::UNAUTHORIZED, security_error(&self.to_string())).into_response()
            }
        }
    }
}

impl IntoResponse for SessionIdValidationError {
    fn into_response(self) -> Response {
        match self {
            SessionIdValidationError::UnAssociatedSessionId
            | SessionIdValidationError::SessionExpired => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
        }
    }
}
