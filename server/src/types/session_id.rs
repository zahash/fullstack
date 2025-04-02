use std::ops::Deref;

use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde_json::json;

use crate::{
    error::{HELP, SECURITY},
    misc::now_iso8601,
    token::Token,
};

#[derive(Debug)]
pub struct SessionId(Token<32>);

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SessionError {
    #[error("`session_id` cookie not found")]
    SessionCookieNotFound,

    #[error("invalid session")]
    InvalidSessionToken,

    #[error("malformed session token")]
    MalformedSessionToken,
}

impl SessionId {
    pub fn new() -> Self {
        Self(Token::new())
    }
}

impl Deref for SessionId {
    type Target = Token<32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&CookieJar> for SessionId {
    type Error = SessionError;

    fn try_from(jar: &CookieJar) -> Result<Self, Self::Error> {
        let value = jar
            .get("session_id")
            .ok_or(SessionError::SessionCookieNotFound)?
            .value();
        let token = Token::try_from(value).map_err(|_| SessionError::MalformedSessionToken)?;
        Ok(SessionId(token))
    }
}

impl TryFrom<&Parts> for SessionId {
    type Error = SessionError;

    fn try_from(parts: &Parts) -> Result<Self, Self::Error> {
        let jar = CookieJar::from_headers(&parts.headers);
        SessionId::try_from(&jar)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for SessionId {
    type Rejection = SessionError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        SessionId::try_from(parts as &Parts)
    }
}

impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        match self {
            SessionError::SessionCookieNotFound | SessionError::InvalidSessionToken => {
                tracing::info!("{:?}", self);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": self.to_string(),
                        "help": HELP,
                        "datetime": now_iso8601()
                    })),
                )
                    .into_response()
            }
            SessionError::MalformedSessionToken => {
                tracing::error!("!SECURITY! {:?}", self);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": self.to_string(),
                        "security": SECURITY
                    })),
                )
                    .into_response()
            }
        }
    }
}
