use axum::{
    Json,
    response::{IntoResponse, Response},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use http::StatusCode;

use crate::core::Credentials;

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

        let bytes = BASE64_STANDARD
            .decode(basic_value)
            .map_err(|_| BasicAuthorizationExtractionError::Base64Decode)?;

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

    #[error("cannot base64 decode :: Authorization: Basic xxx")]
    Base64Decode,

    #[error("Basic Credentials in Authorization header must be utf-8")]
    NonUTF8Credentials,

    #[error("invalid Authorization header format, expected `Basic <base64(username:password)>`")]
    InvalidBasicFormat,
}

impl extra::ErrorKind for BasicAuthorizationExtractionError {
    fn kind(&self) -> &'static str {
        match self {
            BasicAuthorizationExtractionError::NonUTF8HeaderValue => {
                "auth.basic.authorization-header.non-utf8"
            }
            BasicAuthorizationExtractionError::Base64Decode => {
                "auth.basic.authorization-header.base64-decode"
            }
            BasicAuthorizationExtractionError::NonUTF8Credentials => {
                "auth.basic.authorization-header.credentials.non-utf8"
            }
            BasicAuthorizationExtractionError::InvalidBasicFormat => {
                "auth.basic.authorization-header.invalid-format"
            }
        }
    }
}

impl IntoResponse for BasicAuthorizationExtractionError {
    fn into_response(self) -> Response {
        match self {
            BasicAuthorizationExtractionError::NonUTF8HeaderValue
            | BasicAuthorizationExtractionError::NonUTF8Credentials
            | BasicAuthorizationExtractionError::InvalidBasicFormat
            | BasicAuthorizationExtractionError::Base64Decode => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    StatusCode::BAD_REQUEST,
                    Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
        }
    }
}
