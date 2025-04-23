use std::time::Duration;

use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use axum_macros::debug_handler;
use serde::Deserialize;
use time::OffsetDateTime;

use server_core::{
    AccessToken, AppState, Context, InsufficientPermissionsError, InternalError, Permissions,
};

#[derive(Deserialize, Debug)]
pub struct AccessTokenSettings {
    name: String,
    ttl: Option<Duration>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Permission(#[from] InsufficientPermissionsError),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[debug_handler]
#[tracing::instrument(fields(user_id = tracing::field::Empty, ?settings), skip_all)]
pub async fn generate(
    State(AppState { pool, .. }): State<AppState>,
    permissions: Permissions,
    Form(settings): Form<AccessTokenSettings>,
) -> Result<(StatusCode, AccessToken), Error> {
    permissions.require("access_token:create")?;

    let user_id = permissions.user_id();
    tracing::Span::current().record("user_id", &tracing::field::debug(user_id));

    let access_token = AccessToken::new();
    let access_token_hash = access_token.hash();
    let created_at = OffsetDateTime::now_utc();
    let expires_at = settings.ttl.map(|ttl| created_at + ttl);

    sqlx::query!(
            "INSERT INTO access_tokens (name, access_token_hash, user_id, created_at, expires_at) VALUES (?, ?, ?, ?, ?)",
            settings.name,
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

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Permission(err) => err.into_response(),
            Error::Internal(err) => err.into_response(),
        }
    }
}
