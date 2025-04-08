use std::time::Duration;

use anyhow::Context;
use axum::{Form, extract::State, http::StatusCode};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    AppState,
    error::InternalError,
    types::{AccessToken, UserId},
};

#[derive(Deserialize, Debug)]
pub struct AccessTokenSettings {
    ttl: Option<Duration>,
}

#[tracing::instrument(fields(?user_id, ?settings), skip_all)]
pub async fn generate(
    State(AppState { pool, .. }): State<AppState>,
    user_id: UserId,
    Form(settings): Form<AccessTokenSettings>,
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
