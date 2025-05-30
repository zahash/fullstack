use base64::{Engine, prelude::BASE64_STANDARD};
use rand::RngCore;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct Token<const N: usize>([u8; N]);

impl<const N: usize> Token<N> {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let mut buffer = [0u8; N];
        rng.fill_bytes(&mut buffer);
        Self(buffer)
    }

    pub fn hash_sha256(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.0);
        hasher.finalize().to_vec()
    }

    pub fn base64encoded(&self) -> String {
        BASE64_STANDARD.encode(self.0)
    }

    pub fn base64decode(s: &str) -> Result<Self, &str> {
        let bytes = BASE64_STANDARD.decode(s).map_err(|_| s)?;
        let bytes: [u8; N] = bytes.try_into().map_err(|_| s)?;
        Ok(Token(bytes))
    }
}

impl<const N: usize> From<[u8; N]> for Token<N> {
    fn from(bytes: [u8; N]) -> Self {
        Token(bytes)
    }
}
