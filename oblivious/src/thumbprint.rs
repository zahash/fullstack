use std::collections::BTreeMap;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use jose_jwk::Jwk;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn thumbprint(jwk: &Jwk) -> Result<String, ThumbprintError> {
    // use BTreeMap to ensure the keys are sorted lexicographically
    // because hash depends on the order of keys
    let mut map: BTreeMap<&str, Value> = BTreeMap::new();

    match &jwk.key {
        jose_jwk::Key::Ec(ec) => {
            map.insert("crv", serde_json::to_value(&ec.crv)?);
            map.insert("kty", Value::String("EC".to_string()));
            map.insert("x", Value::String(BASE64_URL_SAFE_NO_PAD.encode(&ec.x)));
            map.insert("y", Value::String(BASE64_URL_SAFE_NO_PAD.encode(&ec.y)));
        }
        jose_jwk::Key::Rsa(rsa) => {
            map.insert("e", Value::String(BASE64_URL_SAFE_NO_PAD.encode(&rsa.e)));
            map.insert("kty", Value::String("RSA".to_string()));
            map.insert("n", Value::String(BASE64_URL_SAFE_NO_PAD.encode(&rsa.n)));
        }
        jose_jwk::Key::Okp(okp) => {
            map.insert("crv", serde_json::to_value(&okp.crv)?);
            map.insert("kty", Value::String("OKP".to_string()));
            map.insert("x", Value::String(BASE64_URL_SAFE_NO_PAD.encode(&okp.x)));
        }
        _ => return Err(ThumbprintError::UnsupportedKeyType),
    }

    let thumbprint_json = serde_json::to_string(&map)?;

    let digest = {
        let mut hasher = Sha256::new();
        hasher.update(thumbprint_json.as_bytes());
        hasher.finalize()
    };

    Ok(BASE64_URL_SAFE_NO_PAD.encode(digest))
}

#[derive(Debug, thiserror::Error)]
pub enum ThumbprintError {
    #[error("unsupported key type for generating jwk thumbprint")]
    UnsupportedKeyType,

    #[error("failed to serialize thumbprint components")]
    Json(#[from] serde_json::Error),
}
