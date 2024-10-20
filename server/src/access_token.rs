use std::{ops::Deref, time::Duration};

use anyhow::Context;
use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, Response, StatusCode},
    response::IntoResponse,
    Form,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    error::{AccessTokenError, HandlerError},
    token::Token,
    user_id::UserId,
    AppState,
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

#[derive(Deserialize, Debug)]
pub struct AccessTokenSettings {
    ttl: Option<Duration>,
}

#[debug_handler]
#[tracing::instrument(fields(?user_id, ?settings), skip_all)]
pub async fn generate(
    State(AppState { pool, .. }): State<AppState>,
    user_id: UserId,
    Form(settings): Form<AccessTokenSettings>,
) -> Result<(StatusCode, AccessToken), HandlerError> {
    let access_token = AccessToken::new();
    let access_token_hash = access_token.hash();
    let created_at = OffsetDateTime::now_utc();
    let expires_at = settings.ttl.map(|ttl| created_at + ttl);

    sqlx::query!(
            "INSERT INTO access_tokens (access_token_hash, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)",
            access_token_hash,
            user_id,
            created_at,
            expires_at,
        )
        .execute(&pool)
        .await.context("insert access_token")?;

    tracing::info!(?expires_at, "access_token created");

    Ok((StatusCode::CREATED, access_token))
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
