use axum::{
    Form,
    extract::State,
    http::{HeaderMap, StatusCode, header::USER_AGENT},
    response::{IntoResponse, Response},
    routing::{MethodRouter, post},
};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use bcrypt::verify;
use contextual::Context;
use serde::Deserialize;
use time::{Duration, OffsetDateTime};

use crate::{AppState, core::SessionId};

pub const PATH: &str = "/login";
const COOKIE_DURATION: Duration = Duration::days(30);

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = login::Credentials))]
#[derive(Deserialize)]
pub struct Credentials {
    #[cfg_attr(feature = "openapi", schema(examples("joe")))]
    pub username: String,

    #[cfg_attr(feature = "openapi", schema(examples("h?P7o]37")))]
    pub password: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),

    #[error("{0}")]
    Bcrypt(#[from] contextual::Error<bcrypt::BcryptError>),
}

pub fn method_router() -> MethodRouter<AppState> {
    post(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = PATH,
    operation_id = PATH,
    request_body(
        content = Credentials,
        content_type = "application/x-www-form-urlencoded",
    ),
    responses(
        (status = 200, description = "Login successful, session cookie set"),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
))]
#[debug_handler]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%username), skip_all))]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(Credentials { username, password }): Form<Credentials>,
) -> Result<(CookieJar, StatusCode), Error> {
    #[derive(Debug, Clone)]
    struct User {
        id: i64,
        password_hash: String,
    }

    let user = sqlx::query_as!(
        User,
        r#"SELECT id as "id!", password_hash FROM users WHERE username = ?"#,
        username
    )
    .fetch_optional(&pool)
    .await;

    let user = user
        .context("username -> User { id, password_hash }")?
        .ok_or(Error::InvalidCredentials)?;

    #[cfg(feature = "tracing")]
    tracing::info!("user_id={}", user.id);

    if !verify(password, &user.password_hash).context("verify password hash")? {
        return Err(Error::InvalidCredentials);
    };

    let session_id = SessionId::new();
    let session_id_hash = session_id.hash_sha256();
    let created_at = OffsetDateTime::now_utc();
    let expires_at = created_at + COOKIE_DURATION;
    let user_agent = headers.get(USER_AGENT).and_then(|val| val.to_str().ok());

    sqlx::query!(
        r#"
        INSERT INTO sessions
        (session_id_hash, user_id, created_at, expires_at, user_agent)
        VALUES (?, ?, ?, ?, ?)
        "#,
        session_id_hash,
        user.id,
        created_at,
        expires_at,
        user_agent
    )
    .execute(&pool)
    .await
    .context("insert session")?;

    #[cfg(feature = "tracing")]
    tracing::info!(?expires_at, ?user_agent, "session created");

    let session_cookie = session_id.into_cookie(COOKIE_DURATION);
    let jar = jar.add(session_cookie);

    Ok((jar, StatusCode::OK))
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidCredentials => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                StatusCode::UNAUTHORIZED.into_response()
            }
            Error::Sqlx(_) | Error::Bcrypt(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
