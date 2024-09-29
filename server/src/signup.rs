use anyhow::{anyhow, Context};
use axum::{http::StatusCode, Extension, Form};
use axum_macros::debug_handler;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::{AppError, AuthError};

#[derive(Deserialize)]
pub struct SignUp {
    pub username: String,
    pub password: String,
}

#[debug_handler]
pub async fn signup(
    Extension(pool): Extension<SqlitePool>,
    Form(signup): Form<SignUp>,
) -> Result<StatusCode, AppError> {
    let password_hash =
        bcrypt::hash(signup.password, bcrypt::DEFAULT_COST).context("hash password")?;

    sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        signup.username,
        password_hash,
    )
    .execute(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(e) if e.is_unique_violation() => {
            AuthError::UsernameTaken(signup.username).into()
        }
        e => AppError::Internal(anyhow!(e).context("insert user")),
    })?;

    Ok(StatusCode::CREATED)
}
