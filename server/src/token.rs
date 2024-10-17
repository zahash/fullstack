use std::ops::Deref;

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Token<const N: usize>([u8; N]);

impl<const N: usize> Token<N> {
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

impl<const N: usize> From<[u8; N]> for Token<N> {
    fn from(bytes: [u8; N]) -> Self {
        Token(bytes)
    }
}

impl<'a, const N: usize> TryFrom<&'a str> for Token<N> {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let bytes = BASE64_STANDARD.decode(value).map_err(|_| value)?;
        let bytes: [u8; N] = bytes.try_into().map_err(|_| value)?;
        Ok(Token(bytes))
    }
}

impl<const N: usize> Deref for Token<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> IntoResponse for Token<N> {
    fn into_response(self) -> Response<Body> {
        match Response::builder().body(Body::from(self.base64encoded())) {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("unable to convert {:?} to response :: {:?}", self, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}