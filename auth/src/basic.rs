use base64::{Engine, prelude::BASE64_STANDARD};

use crate::{Base64DecodeError, Credentials};

pub struct Basic {
    pub username: String,
    pub password: String,
}

impl Credentials for Basic {
    type Error = BasicAuthorizationExtractionError;

    fn try_from_headers(headers: &http::HeaderMap) -> Result<Option<Self>, Self::Error> {
        let Some(header_value) = headers.get(http::header::AUTHORIZATION) else {
            return Ok(None);
        };

        let header_value_str = header_value
            .to_str()
            .map_err(|_| BasicAuthorizationExtractionError::NonUTF8HeaderValue)?;

        let Some(basic_value) = header_value_str.strip_prefix("Basic ") else {
            return Ok(None);
        };

        let bytes = BASE64_STANDARD.decode(basic_value).map_err(|_| {
            BasicAuthorizationExtractionError::Base64DecodeError(Base64DecodeError(
                "Authorization: Basic xxx",
            ))
        })?;

        let creds = String::from_utf8(bytes)
            .map_err(|_| BasicAuthorizationExtractionError::NonUTF8Credentials)?;

        let (username, password) = creds
            .split_once(':')
            .ok_or(BasicAuthorizationExtractionError::InvalidBasicFormat)?;
        Ok(Some(Basic {
            username: username.to_string(),
            password: password.to_string(),
        }))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BasicAuthorizationExtractionError {
    #[error("Authorization header value must be utf-8")]
    NonUTF8HeaderValue,

    #[error("{0}")]
    Base64DecodeError(Base64DecodeError),

    #[error("Basic Credentials in Authorization header must be utf-8")]
    NonUTF8Credentials,

    #[error("invalid Authorization header format, expected `Basic <base64(username:password)>`")]
    InvalidBasicFormat,
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for BasicAuthorizationExtractionError {
    fn into_response(self) -> axum::response::Response {
        match self {
            BasicAuthorizationExtractionError::NonUTF8HeaderValue
            | BasicAuthorizationExtractionError::NonUTF8Credentials
            | BasicAuthorizationExtractionError::InvalidBasicFormat => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(extra::json_error_response(self)),
                )
                    .into_response()
            }
            BasicAuthorizationExtractionError::Base64DecodeError(err) => err.into_response(),
        }
    }
}
