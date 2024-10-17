use std::{ops::Deref, time::Duration};

use anyhow::Context;
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Extension, Form,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::{
    error::{AccessTokenError, HandlerError, InternalError},
    request_id::RequestId,
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
        self.0.into_response()
    }
}

#[derive(Deserialize, Debug)]
pub struct AccessTokenSettings {
    ttl: Option<Duration>,
}

#[debug_handler]
#[tracing::instrument(fields(?user_id, ?settings), skip_all)]
pub async fn generate(
    State(state): State<AppState>,
    Extension(request_id): Extension<Option<RequestId>>,
    user_id: UserId,
    Form(settings): Form<AccessTokenSettings>,
) -> Result<(StatusCode, AccessToken), HandlerError> {
    async fn inner(
        pool: SqlitePool,
        user_id: UserId,
        settings: AccessTokenSettings,
    ) -> Result<(StatusCode, AccessToken), InternalError> {
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

    inner(state.pool, user_id, settings)
        .await
        .map_err(|e| HandlerError {
            request_id,
            kind: e.into(),
        })
}

impl TryFrom<&Parts> for AccessToken {
    type Error = AccessTokenError;

    fn try_from(parts: &Parts) -> Result<Self, Self::Error> {
        let header_value = parts
            .headers
            .get("Authorization")
            .ok_or(AccessTokenError::AccessTokenNotFound)?;

        if let Ok(s) = header_value.to_str() {
            if let Some(s) = s.strip_prefix("Token ") {
                if let Ok(token) = Token::<32>::try_from(s) {
                    return Ok(AccessToken(token));
                }
            }
        }

        Err(AccessTokenError::MalformedAccessToken)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AccessToken {
    type Rejection = HandlerError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        AccessToken::try_from(parts as &Parts).map_err(|e| HandlerError {
            request_id: RequestId::from(parts),
            kind: e.into(),
        })
    }
}
