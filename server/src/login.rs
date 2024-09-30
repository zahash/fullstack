use std::time::Duration;

use anyhow::Context;
use axum::{
    http::{header::USER_AGENT, HeaderMap, StatusCode},
    Extension, Form,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use axum_macros::debug_handler;
use bcrypt::verify;
use serde::Deserialize;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{AppError, AuthError};

const DURATION_30_DAYS: Duration = Duration::from_secs(3600 * 24 * 30);

#[derive(Deserialize, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
    pub remember: bool,
}

#[debug_handler]
#[tracing::instrument(fields(username = login.username, remember = login.remember), skip_all)]
pub async fn login(
    Extension(pool): Extension<SqlitePool>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(login): Form<Login>,
) -> Result<(CookieJar, StatusCode), AppError> {
    struct User {
        id: i64,
        password_hash: String,
    }

    let user = sqlx::query_as!(
        User,
        r#"SELECT id as "id!", password_hash FROM users WHERE username = ?"#,
        login.username
    )
    .fetch_optional(&pool)
    .await
    .context("username -> User { id, password_hash }")?
    .ok_or(AuthError::UserNotFound(login.username.clone()))?;

    tracing::info!(user_id = %user.id);

    match verify(login.password, &user.password_hash).context("verify password hash")? {
        false => Err(AuthError::InvalidCredentials.into()),
        true => {
            let session_id = Uuid::new_v4().to_string();
            let created_at = OffsetDateTime::now_utc();
            let expires_at = login.remember.then_some(created_at + DURATION_30_DAYS);
            let user_agent = headers.get(USER_AGENT).and_then(|val| val.to_str().ok());

            sqlx::query!(
                    "INSERT INTO sessions (session_id, user_id, created_at, expires_at, user_agent) VALUES (?, ?, ?, ?, ?)",
                    session_id,
                    user.id,
                    created_at,
                    expires_at,
                    user_agent
                )
                .execute(&pool)
                .await.context("insert session")?;

            tracing::info!(
                session_id = %session_id,
                created_at = %created_at,
                expires_at = %expires_at.map(|t| t.to_string()).unwrap_or("None".into()),
                user_agent = %user_agent.unwrap_or("None"),
                "session created"
            );

            let session_cookie = Cookie::build(("session_id", session_id))
                .path("/")
                .same_site(SameSite::Strict)
                .expires(expires_at)
                .http_only(true)
                .secure(true);
            let jar = jar.add(session_cookie);

            Ok((jar, StatusCode::OK))
        }
    }
}
