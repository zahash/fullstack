use std::ops::Deref;

use axum::{
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

use crate::{
    error::{error, security_error},
    token::Token,
};

#[derive(Clone)]
pub struct AccessToken(Token<32>);

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AccessTokenExtractionError {
    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenHeaderNotFound,

    #[error("invalid access token format. must be in the form 'Token <your-access-token>'")]
    InvalidAccessTokenFormat,

    #[error("malformed access token")]
    MalformedAccessToken,
}

#[derive(thiserror::Error, Debug)]
pub enum AccessTokenValiationError {
    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

    #[error("access token expired")]
    AccessTokenExpired,
}

impl AccessToken {
    pub fn new() -> Self {
        Self(Token::new())
    }
}

impl Deref for AccessToken {
    type Target = Token<32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoResponse for AccessToken {
    fn into_response(self) -> Response {
        self.base64encoded().into_response()
    }
}

impl TryFrom<&Parts> for AccessToken {
    type Error = AccessTokenExtractionError;

    fn try_from(parts: &Parts) -> Result<Self, Self::Error> {
        let header_value = parts
            .headers
            .get("Authorization")
            .ok_or(AccessTokenExtractionError::AccessTokenHeaderNotFound)?;

        let token_str = header_value
            .to_str()
            .ok()
            .and_then(|s| s.strip_prefix("Token "))
            .ok_or(AccessTokenExtractionError::InvalidAccessTokenFormat)?;

        Token::base64decode(token_str)
            .map(|token| AccessToken(token))
            .map_err(|_| AccessTokenExtractionError::MalformedAccessToken)
    }
}

// impl<S: Send + Sync> FromRequestParts<S> for AccessToken {
//     type Rejection = AccessTokenExtractionError;

//     async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
//         AccessToken::try_from(parts as &Parts)
//     }
// }

impl IntoResponse for AccessTokenExtractionError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenExtractionError::AccessTokenHeaderNotFound
            | AccessTokenExtractionError::InvalidAccessTokenFormat => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            AccessTokenExtractionError::MalformedAccessToken => {
                tracing::error!("!SECURITY! {:?}", self);
                (StatusCode::UNAUTHORIZED, security_error(&self.to_string())).into_response()
            }
        }
    }
}

impl IntoResponse for AccessTokenValiationError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenValiationError::UnAssociatedAccessToken
            | AccessTokenValiationError::AccessTokenExpired => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
        }
    }
}
