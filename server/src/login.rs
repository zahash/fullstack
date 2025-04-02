use std::time::Duration;

use axum::{
    extract::State,
    http::{header::USER_AGENT, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Form,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use bcrypt::verify;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    error::{Context, InternalError},
    types::{SessionId, UserId, Username},
    AppState,
};

const DURATION_30_DAYS: Duration = Duration::from_secs(3600 * 24 * 30);

#[derive(Deserialize)]
pub struct Login {
    pub username: Username,
    pub password: String, // `Password` type not necessary because no checks are required
    pub remember: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[tracing::instrument(fields(?username, ?remember), skip_all)]
pub async fn login<T>(
    State(AppState { pool, .. }): State<AppState<T>>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(Login {
        username,
        password,
        remember,
    }): Form<Login>,
) -> Result<(CookieJar, StatusCode), Error> {
    struct User {
        id: UserId,
        password_hash: String,
    }

    let user = sqlx::query_as!(
        User,
        r#"SELECT id as "id!", password_hash FROM users WHERE username = ?"#,
        username
    )
    .fetch_optional(&pool)
    .await
    .context("username -> User { id, password_hash }")?
    .ok_or(Error::InvalidCredentials)?;

    tracing::info!("{:?}", user.id);

    match verify(password, &user.password_hash).context("verify password hash")? {
        false => Err(Error::InvalidCredentials),
        true => {
            let session_id = SessionId::new();
            let session_id_hash = session_id.hash();
            let created_at = OffsetDateTime::now_utc();
            let expires_at = remember.then_some(created_at + DURATION_30_DAYS);
            let user_agent = headers.get(USER_AGENT).and_then(|val| val.to_str().ok());

            sqlx::query!(
                    "INSERT INTO sessions (session_id_hash, user_id, created_at, expires_at, user_agent) VALUES (?, ?, ?, ?, ?)",
                    session_id_hash,
                    user.id,
                    created_at,
                    expires_at,
                    user_agent
                )
                .execute(&pool)
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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidCredentials => {
                tracing::info!("{:?}", self);
                StatusCode::UNAUTHORIZED.into_response()
            }
            Error::Internal(err) => err.into_response(),
        }
    }
}
