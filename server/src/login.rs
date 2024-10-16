use std::time::Duration;

use anyhow::Context;
use axum::{
    extract::State,
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

use crate::{
    error::{AuthError, HandlerError, HandlerErrorKind},
    request_id::RequestId,
    session_id::SessionId,
    user_id::UserId,
    AppState,
};

const DURATION_30_DAYS: Duration = Duration::from_secs(3600 * 24 * 30);

#[derive(Deserialize, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
    pub remember: bool,
}

#[debug_handler]
#[tracing::instrument(fields(?login), skip_all)]
pub async fn login(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(login): Form<Login>,
) -> Result<(CookieJar, StatusCode), HandlerError> {
    async fn inner(
        pool: &SqlitePool,
        headers: HeaderMap,
        jar: CookieJar,
        login: Login,
    ) -> Result<(CookieJar, StatusCode), HandlerErrorKind> {
        struct User {
            id: UserId,
            password_hash: String,
        }

        let user = sqlx::query_as!(
            User,
            r#"SELECT id as "id!", password_hash FROM users WHERE username = ?"#,
            login.username
        )
        .fetch_optional(pool)
        .await
        .context("username -> User { id, password_hash }")?
        .ok_or(AuthError::UserNotFound(login.username.clone()))?;

        tracing::info!("{:?}", user.id);

        match verify(login.password, &user.password_hash).context("verify password hash")? {
            false => Err(AuthError::InvalidCredentials.into()),
            true => {
                let session_id = SessionId::new();
                let session_id_hash = session_id.hash();
                let created_at = OffsetDateTime::now_utc();
                let expires_at = login.remember.then_some(created_at + DURATION_30_DAYS);
                let user_agent = headers.get(USER_AGENT).and_then(|val| val.to_str().ok());

                sqlx::query!(
                    "INSERT INTO sessions (session_id_hash, user_id, created_at, expires_at, user_agent) VALUES (?, ?, ?, ?, ?)",
                    session_id_hash,
                    user.id,
                    created_at,
                    expires_at,
                    user_agent
                )
                .execute(pool)
                .await.context("insert session")?;

                tracing::info!(?expires_at, ?user_agent, "session created");

                let session_cookie = Cookie::build(("session_id", session_id.base64encoded()))
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

    inner(&state.pool, headers, jar, login)
        .await
        .map_err(|e| HandlerError {
            request_id,
            kind: e.into(),
        })
}
