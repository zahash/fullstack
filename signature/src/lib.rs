use std::{convert::TryFrom, time::Duration};

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use contextual::Context;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{
    Sha256,
    digest::{InvalidLength, MacError},
};
use time::OffsetDateTime;

/// A generic container for a token that is signed and has an expiration date.
/// The token is signed using HMAC-SHA256.
#[derive(Debug, Clone)]
pub struct Signed<T> {
    header: Header,
    token: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Header {
    /// issued at time
    iat: OffsetDateTime,
    /// expiry time
    exp: OffsetDateTime,
}

impl<T> Signed<T> {
    const DEFAULT_TTL: Duration = Duration::from_secs(3600);

    /// Creates a new `Signed` token with the default settings.
    /// Settings can be modified using the various builder-style methods.
    pub fn new(token: T) -> Self {
        let iat = OffsetDateTime::now_utc();
        let exp = iat + Self::DEFAULT_TTL;
        let header = Header { iat, exp };
        Signed { header, token }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.header.exp = self.header.iat + ttl;
        self
    }

    pub fn token(self) -> Result<T, TemporalValidityError> {
        let now = OffsetDateTime::now_utc();

        if self.header.exp < now {
            return Err(TemporalValidityError::Expired {
                exp: self.header.exp,
                now,
            });
        }

        if self.header.iat > now {
            return Err(TemporalValidityError::NotYetValid {
                iat: self.header.iat,
                now,
            });
        }

        Ok(self.token)
    }

    /// Encodes the `Signed` token into a url-safe base64 encoded string with no padding.
    pub fn encode(&self, secret: &[u8]) -> Result<String, EncodeError>
    where
        T: AsRef<[u8]>,
    {
        let header_json = serde_json::to_string(&self.header).context("header")?;
        let header_base64encoded = BASE64_URL_SAFE_NO_PAD.encode(header_json);
        let token_base64encoded = BASE64_URL_SAFE_NO_PAD.encode(self.token.as_ref());
        let signature_base64encoded = {
            let signing_input = format!("{header_base64encoded}.{token_base64encoded}");
            let mut mac = Hmac::<Sha256>::new_from_slice(secret)
                .map_err(|_: InvalidLength| EncodeError::InvalidKeyLength)?;
            mac.update(signing_input.as_bytes());
            let signature_bytes = mac.finalize().into_bytes();
            BASE64_URL_SAFE_NO_PAD.encode(signature_bytes)
        };
        Ok(format!(
            "{header_base64encoded}.{token_base64encoded}.{signature_base64encoded}"
        ))
    }

    /// Decodes a `Signed` token from a url-safe base64 encoded string with no padding.
    pub fn decode(
        s: &str,
        secret: &[u8],
    ) -> Result<Self, DecodeError<<T as TryFrom<Vec<u8>>>::Error>>
    where
        T: TryFrom<Vec<u8>>,
        <T as TryFrom<Vec<u8>>>::Error: std::error::Error,
    {
        let parts = s.split('.').collect::<Vec<&str>>();

        match parts.as_slice() {
            [header_part, token_part, signature_part] => {
                let signature = BASE64_URL_SAFE_NO_PAD
                    .decode(signature_part)
                    .context("signature")?;
                let mut mac = Hmac::<Sha256>::new_from_slice(secret)
                    .map_err(|InvalidLength| DecodeError::InvalidKeyLength)?;
                mac.update(format!("{header_part}.{token_part}").as_bytes());
                mac.verify_slice(&signature)?;

                let header = {
                    let bytes = BASE64_URL_SAFE_NO_PAD
                        .decode(header_part)
                        .context("header")?;

                    let json_string =
                        String::from_utf8(bytes).map_err(|_| DecodeError::NonUTF8("header"))?;

                    serde_json::from_str::<Header>(&json_string).context("header")?
                };

                let token = {
                    let bytes = BASE64_URL_SAFE_NO_PAD.decode(token_part).context("token")?;
                    T::try_from(bytes).map_err(DecodeError::TokenFromBytes)?
                };

                Ok(Self { header, token })
            }
            _ => Err(DecodeError::InvalidFormat),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TemporalValidityError {
    #[error("token expired at {exp} (now: {now})")]
    Expired {
        exp: OffsetDateTime,
        now: OffsetDateTime,
    },

    #[error("token not valid yet; issued at {iat} (now: {now})")]
    NotYetValid {
        iat: OffsetDateTime,
        now: OffsetDateTime,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum EncodeError {
    #[error("Invalid Key Length")]
    InvalidKeyLength,

    #[error("{0}")]
    Serde(#[from] contextual::Error<serde_json::Error>),
}

#[derive(thiserror::Error, Debug)]
pub enum DecodeError<E>
where
    E: std::error::Error + 'static,
{
    #[error("Invalid token format")]
    InvalidFormat,

    #[error("Invalid Key Length")]
    InvalidKeyLength,

    #[error("{0}")]
    MacMismatch(#[from] MacError),

    #[error("Non-UTF8 sequence for {0}")]
    NonUTF8(&'static str),

    #[error("{0}")]
    Serde(#[from] contextual::Error<serde_json::Error>),

    #[error("{0}")]
    Base64(#[from] contextual::Error<base64::DecodeError>),

    #[error("failed to build token from bytes")]
    TokenFromBytes(#[source] E),
}
