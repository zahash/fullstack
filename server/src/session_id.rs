use std::ops::Deref;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::{
    error::{HandlerError, SessionError},
    request_id::RequestId,
    token::Token,
};

pub struct SessionId(Token<32>);

impl SessionId {
    pub fn new() -> Self {
        Self(Token::<32>::new())
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
        let token = Token::<32>::try_from(value).map_err(|_| SessionError::MalformedSessionToken)?;
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

#[async_trait]
impl<S> FromRequestParts<S> for SessionId {
    type Rejection = HandlerError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        SessionId::try_from(parts as &Parts).map_err(|e| HandlerError {
            request_id: RequestId::from(parts),
            kind: e.into(),
        })
    }
}
