use std::ops::Deref;

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::{Base64DecodeError, Token, UserId, Valid, error};

pub struct SessionId(Token<32>);

pub struct SessionInfo {
    pub user_id: UserId,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub user_agent: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionValidationError {
    #[error("session expired")]
    SessionExpired,
}

impl SessionId {
    pub fn new() -> Self {
        Self(Token::new())
    }

    pub fn into_cookie<'a>(self, expires_at: OffsetDateTime) -> Cookie<'a> {
        Cookie::build(("session_id", self.base64encoded()))
            .path("/")
            .same_site(SameSite::Strict)
            .expires(expires_at)
            .http_only(true)
            .secure(true)
            .build()
    }

    pub fn try_from_cookie_jar(jar: &CookieJar) -> Result<Option<SessionId>, Base64DecodeError> {
        let Some(session_cookie) = jar.get("session_id") else {
            return Ok(None);
        };

        let token = Token::base64decode(session_cookie.value())
            .map_err(|_| Base64DecodeError("SessionId"))?;

        Ok(Some(SessionId(token)))
    }

    pub fn try_from_headers(headers: &HeaderMap) -> Result<Option<SessionId>, Base64DecodeError> {
        let jar = CookieJar::from_headers(headers);
        SessionId::try_from_cookie_jar(&jar)
    }

    pub async fn info(&self, pool: &SqlitePool) -> Result<Option<SessionInfo>, sqlx::Error> {
        let session_id_hash = self.hash();

        sqlx::query_as!(
            SessionInfo,
            "SELECT user_id, created_at, expires_at, user_agent FROM sessions WHERE session_id_hash = ?",
            session_id_hash
        )
        .fetch_optional(pool)
        .await
    }
}

impl SessionInfo {
    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }

    pub fn validate(self) -> Result<Valid<SessionInfo>, SessionValidationError> {
        if self.is_expired() {
            return Err(SessionValidationError::SessionExpired);
        }

        Ok(Valid(self))
    }
}

impl Valid<SessionInfo> {
    pub async fn permissions(&self, pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
        let user_id = &self.0.user_id;

        let permissions = sqlx::query_scalar!(
            "SELECT p.permission from permissions p
             INNER JOIN user_permissions up ON up.permission_id = p.id
             WHERE up.user_id = ?",
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(permissions)
    }
}

pub trait SessionExt {
    fn remove_session_cookie(self) -> Self;
}

impl SessionExt for CookieJar {
    fn remove_session_cookie(self) -> Self {
        self.remove("session_id")
    }
}

impl Deref for SessionId {
    type Target = Token<32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoResponse for SessionValidationError {
    fn into_response(self) -> Response {
        match self {
            SessionValidationError::SessionExpired => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
        }
    }
}
