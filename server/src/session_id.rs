use axum_extra::extract::CookieJar;
use base64::{prelude::BASE64_STANDARD, Engine};

use crate::{
    error::{AuthError, CookieError, PublicError},
    token::Token,
};

pub type SessionId = Token<32>;

impl TryFrom<&CookieJar> for SessionId {
    type Error = PublicError;

    fn try_from(jar: &CookieJar) -> Result<Self, Self::Error> {
        let value = jar
            .get("session_id")
            .ok_or(CookieError::CookieNotFound("session_id"))?
            .value();
        let bytes = BASE64_STANDARD
            .decode(value)
            .map_err(|_| AuthError::InvalidSession)?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| AuthError::InvalidSession)?;
        Ok(SessionId::from(bytes))
    }
}
