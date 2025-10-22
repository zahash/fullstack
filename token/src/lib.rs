use std::fmt::Display;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct Token<const N: usize>(Zeroizing<[u8; N]>);

impl<const N: usize> Token<N> {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let mut buffer = [0u8; N];
        rng.fill_bytes(&mut buffer);
        Self(Zeroizing::new(buffer))
    }

    pub fn hash_sha256(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.0);
        hasher.finalize().to_vec()
    }

    pub fn base64encoded(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(&self.0)
    }

    pub fn base64decode(s: &str) -> Result<Self, &str> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(s).map_err(|_| s)?;
        let bytes: [u8; N] = bytes.try_into().map_err(|_| s)?;
        Ok(Self(Zeroizing::new(bytes)))
    }

    #[inline]
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        Token::from(bytes)
    }

    #[inline]
    pub fn into_bytes(self) -> Zeroizing<[u8; N]> {
        self.into()
    }
}

impl<const N: usize> From<[u8; N]> for Token<N> {
    #[inline]
    fn from(bytes: [u8; N]) -> Self {
        Self(Zeroizing::new(bytes))
    }
}

impl<const N: usize> From<Token<N>> for Zeroizing<[u8; N]> {
    #[inline]
    fn from(token: Token<N>) -> Self {
        token.0
    }
}

impl<const N: usize> Display for Token<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({})", self.base64encoded())
    }
}
