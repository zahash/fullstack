use auth::SessionId;
use axum::{
    Form,
    extract::State,
    http::{HeaderMap, StatusCode, header::USER_AGENT},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use bcrypt::verify;
use contextual::Context;
use dashcache::DashCache;
use serde::Deserialize;
use tag::Tag;
use time::{Duration, OffsetDateTime};

use crate::AppState;

const COOKIE_DURATION: Duration = Duration::days(30);

#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("{0:?}")]
    DataAccess(#[from] contextual::Error<data_access::Error>),

    #[error("{0:?}")]
    Bcrypt(#[from] contextual::Error<bcrypt::BcryptError>),
}

#[debug_handler]
#[tracing::instrument(fields(?username), skip_all)]
pub async fn login(
    State(AppState { data_access, .. }): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(Login { username, password }): Form<Login>,
) -> Result<(CookieJar, StatusCode), Error> {
    #[derive(Debug, Clone)]
    struct User {
        id: i64,
        password_hash: String,
    }

    let user = data_access
        .read(
            |pool| {
                sqlx::query_as!(
                    User,
                    r#"SELECT id as "id!", password_hash FROM users WHERE username = ?"#,
                    username
                )
                .fetch_optional(pool)
            },
            "login_user__from__username",
            username.clone(),
            |value| match value {
                Some(user) => vec![Tag {
                    table: "users",
                    primary_key: Some(user.id),
                }],
                None => vec![Tag {
                    table: "users",
                    primary_key: None,
                }],
            },
            DashCache::new,
        )
        .await;

    let user = user
        .context("username -> User { id, password_hash }")?
        .ok_or(Error::InvalidCredentials)?;

    tracing::info!("{:?}", user.id);

    if !verify(password, &user.password_hash).context("verify password hash")? {
        return Err(Error::InvalidCredentials);
    };

    let session_id = SessionId::new();
    let session_id_hash = session_id.hash_sha256();
    let created_at = OffsetDateTime::now_utc();
    let expires_at = created_at + COOKIE_DURATION;
    let user_agent = headers.get(USER_AGENT).and_then(|val| val.to_str().ok());

    data_access
        .write(
            |pool| {
                sqlx::query!(
                    r#"
                    INSERT INTO sessions
                    (session_id_hash, user_id, created_at, expires_at, user_agent)
                    VALUES (?, ?, ?, ?, ?)
                    RETURNING id as "id!"
                    "#,
                    session_id_hash,
                    user.id,
                    created_at,
                    expires_at,
                    user_agent
                )
                .fetch_one(pool)
            },
            |value| {
                vec![
                    Tag {
                        table: "sessions",
                        primary_key: None,
                    },
                    Tag {
                        table: "sessions",
                        primary_key: Some(value.id),
                    },
                ]
            },
        )
        .await
        .context("insert session")?;

    tracing::info!(?expires_at, ?user_agent, "session created");

    let session_cookie = session_id.into_cookie(COOKIE_DURATION);
    let jar = jar.add(session_cookie);

    Ok((jar, StatusCode::OK))
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidCredentials => {
                tracing::info!("{:?}", self);
                StatusCode::UNAUTHORIZED.into_response()
            }
            Error::DataAccess(_) | Error::Bcrypt(_) => {
                tracing::error!("{:?}", self);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
