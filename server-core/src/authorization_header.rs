use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD};

use crate::{AccessToken, Base64DecodeError, Token, error};

#[derive(thiserror::Error, Debug)]
pub enum AuthorizationHeaderError {
    #[error("Authorization header value must be utf-8")]
    NonUTF8HeaderValue,

    #[error("invalid Authorization type, only `Basic` and `Token` are allowed")]
    InvalidAuthorizationType,

    #[error("invalid Authorization header format, expected `Basic <base64(username:password)>`")]
    InvalidBasicFormat,

    #[error("{0}")]
    Base64Decode(Base64DecodeError),
}

pub enum AuthorizationHeader {
    AccessToken(AccessToken),
    Basic { username: String, password: String },
}

impl AuthorizationHeader {
    pub fn try_from_headers(
        headers: &HeaderMap,
    ) -> Result<Option<AuthorizationHeader>, AuthorizationHeaderError> {
        let Some(header_value) = headers.get("Authorization") else {
            return Ok(None);
        };

        let header_value_str = header_value
            .to_str()
            .map_err(|_| AuthorizationHeaderError::NonUTF8HeaderValue)?;

        if let Some(value) = header_value_str.strip_prefix("Token ") {
            let token = Token::base64decode(value).map_err(|_| {
                AuthorizationHeaderError::Base64Decode(Base64DecodeError("Authorization header"))
            })?;
            return Ok(Some(AuthorizationHeader::AccessToken(AccessToken::from(
                token,
            ))));
        }

        if let Some(value) = header_value_str.strip_prefix("Basic ") {
            let bytes = BASE64_STANDARD.decode(value).map_err(|_| {
                AuthorizationHeaderError::Base64Decode(Base64DecodeError("Authorization header"))
            })?;
            let creds = String::from_utf8(bytes)
                .map_err(|_| AuthorizationHeaderError::NonUTF8HeaderValue)?;
            let (username, password) = creds
                .split_once(':')
                .ok_or(AuthorizationHeaderError::InvalidBasicFormat)?;

            return Ok(Some(AuthorizationHeader::Basic {
                username: username.to_string(),
                password: password.to_string(),
            }));
        }

        Err(AuthorizationHeaderError::InvalidAuthorizationType)
    }
}

impl IntoResponse for AuthorizationHeaderError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthorizationHeaderError::NonUTF8HeaderValue
            | AuthorizationHeaderError::InvalidAuthorizationType
            | AuthorizationHeaderError::InvalidBasicFormat
            | AuthorizationHeaderError::Base64Decode(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
        }
    }
}
