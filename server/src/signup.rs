use std::sync::LazyLock;

use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Extension, Form};
use axum_macros::debug_handler;
use compiletime_regex::regex;
use regex::Regex;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    error::{AuthError, HandlerError, HandlerErrorKind},
    request_id::RequestId,
    AppState,
};

#[derive(Deserialize)]
pub struct SignUp {
    pub username: String,
    pub password: String,
}

#[debug_handler]
#[tracing::instrument(fields(username = %signup.username), skip_all, ret)]
pub async fn signup(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Form(signup): Form<SignUp>,
) -> Result<StatusCode, HandlerError> {
    async fn inner(
        pool: &SqlitePool,
        SignUp { username, password }: SignUp,
    ) -> Result<StatusCode, HandlerErrorKind> {
        let password_hash =
            bcrypt::hash(password, bcrypt::DEFAULT_COST).context("hash password")?;

        sqlx::query!(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            username,
            password_hash,
        )
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => {
                AuthError::UsernameTaken(username).into()
            }
            e => <anyhow::Error as Into<HandlerErrorKind>>::into(anyhow!(e).context("insert user")),
        })?;

        Ok(StatusCode::CREATED)
    }

    inner(&state.pool, signup).await.map_err(|e| HandlerError {
        request_id,
        kind: e.into(),
    })
}

#[derive(Deserialize)]
pub struct CheckUsernameAvailability {
    pub username: String,
}

#[debug_handler]
#[tracing::instrument(fields(%username), skip_all, ret)]
pub async fn check_username_availability(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Form(CheckUsernameAvailability { username }): Form<CheckUsernameAvailability>,
) -> Result<StatusCode, HandlerError> {
    async fn inner(pool: &SqlitePool, username: String) -> Result<StatusCode, HandlerErrorKind> {
        let username = validate_username(username)?;

        let username_exists = match sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ? LIMIT 1) as username_exists",
            username
        )
        .fetch_one(pool)
        .await
        .context("check_username_availability")?
        .username_exists
        {
            0 => false,
            _ => true,
        };

        match username_exists {
            true => Err(<AuthError as Into<HandlerErrorKind>>::into(
                AuthError::UsernameTaken(username),
            )),
            false => Ok(StatusCode::OK),
        }
    }

    inner(&state.pool, username)
        .await
        .map_err(|e| HandlerError {
            request_id,
            kind: e.into(),
        })
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
