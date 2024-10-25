use std::ops::Deref;

use axum::{
    async_trait,
    body::Body,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    error::{HELP, SECURITY},
    misc::now_iso8601,
    token::Token,
};

#[derive(Debug)]
pub struct AccessToken(Token<32>);

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AccessTokenError {
    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenNotFound,

    // could be because it expired or it was not found.
    #[error("invalid access token")]
    InvalidAccessToken,

    #[error("invalid access token format. must be in the form 'Token <your-access-token>'")]
    InvalidAccessTokenFormat,

    #[error("malformed access token")]
    MalformedAccessToken,
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
        match Response::builder().body(Body::from(self.base64encoded())) {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("unable to convert {:?} to response :: {:?}", self, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl TryFrom<&Parts> for AccessToken {
    type Error = AccessTokenError;

    fn try_from(parts: &Parts) -> Result<Self, Self::Error> {
        let header_value = parts
            .headers
            .get("Authorization")
            .ok_or(AccessTokenError::AccessTokenNotFound)?;

        let token_str = header_value
            .to_str()
            .ok()
            .and_then(|s| s.strip_prefix("Token "))
            .ok_or(AccessTokenError::InvalidAccessTokenFormat)?;

        Token::try_from(token_str)
            .map(|token| AccessToken(token))
            .map_err(|_| AccessTokenError::MalformedAccessToken)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AccessToken {
    type Rejection = AccessTokenError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        AccessToken::try_from(parts as &Parts)
    }
}

impl IntoResponse for AccessTokenError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenError::AccessTokenNotFound
            | AccessTokenError::InvalidAccessToken
            | AccessTokenError::InvalidAccessTokenFormat => {
                tracing::info!("{:?}", self);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": self.to_string(),
                        "help": HELP,
                        "datetime": now_iso8601()
                    })),
                )
                    .into_response()
            }
            AccessTokenError::MalformedAccessToken => {
                tracing::error!("!SECURITY! {:?}", self);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": self.to_string(),
                        "security": SECURITY
                    })),
                )
                    .into_response()
            }
        }
    }
}
