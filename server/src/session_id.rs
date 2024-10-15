use axum_extra::extract::CookieJar;
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

use crate::error::{AuthError, CookieError, PublicError};

const N: usize = 32;

pub struct SessionId([u8; N]);

impl SessionId {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let mut buffer = [0u8; N];
        rng.fill_bytes(&mut buffer);
        Self(buffer)
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.0);
        hasher.finalize().to_vec()
    }

    pub fn base64encoded(&self) -> String {
        BASE64_STANDARD.encode(self.0)
    }
}

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
        let bytes: [u8; N] = bytes.try_into().map_err(|_| AuthError::InvalidSession)?;
        Ok(SessionId(bytes))
    }
}
