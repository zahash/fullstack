use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::{
    error::error,
    token::Token,
    types::{AccessToken, Base64DecodeError},
};

#[derive(thiserror::Error, Debug)]
pub enum AuthorizationHeaderError {
    // #[error("Authorization header not found")]
    // HeaderNotFound,
    #[error("Authorization header value must be utf-8")]
    NonUTF8HeaderValue,

    #[error("invalid Authorization type, only `Basic` and `Token` are allowed")]
    InvalidAuthorizationType,

    #[error("{0}")]
    Base64Decode(Base64DecodeError),
}

pub enum AuthorizationHeader {
    AccessToken(AccessToken),
}

// impl TryFrom<&HeaderMap> for AuthorizationHeader {
//     type Error = AuthorizationHeaderError;

//     fn try_from(headers: &HeaderMap) -> Result<Self, Self::Error> {
//         let header_value = headers
//             .get("Authorization")
//             .ok_or(AuthorizationHeaderError::HeaderNotFound)?
//             .to_str()
//             .map_err(|_| AuthorizationHeaderError::NonUTF8HeaderValue)?;

//         if let Some(value) = header_value.strip_prefix("Token ") {
//             let token =
//                 Token::base64decode(value).map_err(|_| AuthorizationHeaderError::Base64Decode)?;
//             return Ok(AuthorizationHeader::AccessToken(AccessToken::from(token)));
//         }

//         Err(AuthorizationHeaderError::InvalidAuthorizationType)
//     }
// }

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

        Err(AuthorizationHeaderError::InvalidAuthorizationType)
    }
}

impl IntoResponse for AuthorizationHeaderError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthorizationHeaderError::NonUTF8HeaderValue
            | AuthorizationHeaderError::InvalidAuthorizationType
            | AuthorizationHeaderError::Base64Decode(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
        }
    }
}
