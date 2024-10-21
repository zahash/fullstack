use std::sync::LazyLock;

use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use compiletime_regex::regex;
use regex::Regex;
use serde::Deserialize;

use crate::{
    error::{AuthError, HandlerError},
    AppState,
};

#[derive(Deserialize)]
pub struct Username {
    pub username: String,
}

#[debug_handler]
#[tracing::instrument(fields(?username), skip_all, ret)]
pub async fn username_availability(
    State(AppState { pool, .. }): State<AppState>,
    Query(Username { username }): Query<Username>,
) -> Result<StatusCode, HandlerError> {
    let username = validate_username(username)?;

    let username_exists = match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = ? LIMIT 1) as username_exists",
        username
    )
    .fetch_one(&pool)
    .await
    .context("check_username_availability")?
    .username_exists
    {
        0 => false,
        _ => true,
    };

    match username_exists {
        true => Err(<AuthError as Into<HandlerError>>::into(
            AuthError::UsernameTaken(username),
        )),
        false => Ok(StatusCode::OK),
    }
}

const RE_USERNAME: LazyLock<Regex> = LazyLock::new(|| regex!(r#"^[A-Za-z0-9_]{2,30}$"#));

fn validate_username(username: String) -> Result<String, AuthError> {
    match RE_USERNAME.is_match(&username) {
        true => Ok(username),
        false => Err(AuthError::InvalidUsername {
            username,
            reason: "must be between 2-30 in length. must only contain `A-Z` `a-z` `0-9` and `_`"
                .into(),
        }),
    }
}
