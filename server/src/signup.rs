use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Form};
use axum_macros::debug_handler;
use serde::Deserialize;

use crate::{
    error::{AuthError, HandlerError},
    AppState,
};

#[derive(Deserialize)]
pub struct SignUp {
    pub username: String,
    pub password: String,
}

#[debug_handler]
#[tracing::instrument(fields(?username), skip_all, ret)]
pub async fn signup(
    State(AppState { pool, .. }): State<AppState>,
    Form(SignUp { username, password }): Form<SignUp>,
) -> Result<StatusCode, HandlerError> {
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).context("hash password")?;

    sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        username,
        password_hash,
    )
    .execute(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(e) if e.is_unique_violation() => {
            AuthError::UsernameTaken(username).into()
        }
        e => <anyhow::Error as Into<HandlerError>>::into(anyhow!(e).context("insert user")),
    })?;

    Ok(StatusCode::CREATED)
}
