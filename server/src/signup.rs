use std::sync::LazyLock;

use anyhow::{anyhow, Context};
use axum::{http::StatusCode, Extension, Form};
use axum_macros::debug_handler;
use regex::Regex;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    error::{HandlerError, AuthError, ErrorKind, RequestIdCtx},
    types::RequestId,
};

#[derive(Deserialize)]
pub struct SignUpReq {
    pub username: String,
    pub password: String,
}

#[debug_handler]
#[tracing::instrument(fields(username = username), skip_all, ret)]
pub async fn signup(
    Extension(request_id): Extension<RequestId>,
    Extension(pool): Extension<SqlitePool>,
    Form(SignUpReq { username, password }): Form<SignUpReq>,
) -> Result<StatusCode, HandlerError> {
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .context("hash password")
        .request_id(request_id.clone())?;

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
        e => ErrorKind::Internal(anyhow!(e).context("insert user")),
    })
    .request_id(request_id)?;

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
pub struct CheckUsernameReq {
    pub username: String,
}

#[debug_handler]
#[tracing::instrument(fields(username = username), skip_all, ret)]
pub async fn check_username_availability(
    Extension(request_id): Extension<RequestId>,
    Extension(pool): Extension<SqlitePool>,
    Form(CheckUsernameReq { username }): Form<CheckUsernameReq>,
) -> Result<StatusCode, HandlerError> {
    let username = validate_username(username).request_id(request_id.clone())?;

    let username_exists = match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = ? LIMIT 1) as username_exists",
        username
    )
    .fetch_one(&pool)
    .await
    .context("check_username_availability")
    .request_id(request_id.clone())?
    .username_exists
    {
        0 => false,
        _ => true,
    };

    match username_exists {
        true => Err(<AuthError as Into<ErrorKind>>::into(
            AuthError::UsernameTaken(username),
        )),
        false => Ok(StatusCode::OK),
    }
    .request_id(request_id)
}

const USERNAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^[A-Za-z0-9_]{2,30}$"#).unwrap());

#[inline]
fn validate_username(username: String) -> Result<String, AuthError> {
    match USERNAME_REGEX.is_match(&username) {
        true => Ok(username),
        false => Err(AuthError::InvalidUsername {
            username,
            reason: "must be between 2-30 in length. must only contain `A-Z` `a-z` `0-9` and `_`"
                .into(),
        }),
    }
}
