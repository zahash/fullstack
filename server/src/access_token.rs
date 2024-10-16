use std::time::Duration;

use anyhow::Context;
use axum::{extract::State, http::StatusCode, Extension, Form};
use axum_macros::debug_handler;
use serde::Deserialize;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::{
    error::{HandlerError, InternalError},
    request_id::RequestId,
    token::Token,
    user_id::UserId,
    AppState,
};

pub type AccessToken = Token<32>;

#[derive(Deserialize, Debug)]
pub struct AccessTokenSettings {
    ttl: Option<Duration>,
}

#[debug_handler]
#[tracing::instrument(fields(?user_id, ?settings), skip_all)]
pub async fn generate(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
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
