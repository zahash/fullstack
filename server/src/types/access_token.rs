use std::ops::Deref;

use axum::{
    async_trait,
    body::Body,
    extract::FromRequestParts,
    http::{request::Parts, Response, StatusCode},
    response::IntoResponse,
};

use crate::{
    error::{AccessTokenError, HandlerError},
    token::Token,
};

#[derive(Debug)]
pub struct AccessToken(Token<32>);

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
    fn into_response(self) -> axum::response::Response {
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
    type Rejection = HandlerError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        AccessToken::try_from(parts as &Parts).map_err(HandlerError::from)
    }
}
