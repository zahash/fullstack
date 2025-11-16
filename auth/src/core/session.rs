use std::ops::Deref;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use cookie::{Cookie, SameSite, time::Duration};
use http::{StatusCode, header::COOKIE};
use time::OffsetDateTime;
use token::Token;

use crate::core::{Credentials, Permission, Verified, permission::Authorizable};

const SESSION_ID: &str = "session_id";

pub struct SessionId(Token<32>);

impl Credentials for SessionId {
    type Error = SessionCookieExtractionError;

    fn try_from_headers(headers: &http::HeaderMap) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        Ok(headers
            .get_all(COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(';'))
            .map(|value| value.trim())
            .filter_map(|cookie_str| Cookie::parse(cookie_str).ok())
            .find(|cookie| cookie.name() == SESSION_ID)
            .map(|cookie| {
                Token::base64decode(cookie.value())
                    .map_err(|_| SessionCookieExtractionError::Base64Decode)
            })
            .transpose()?
            .map(SessionId))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SessionCookieExtractionError {
    #[error("cannot base64 decode :: Session Cookie")]
    Base64Decode,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user_id: i64,
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
        Self(Token::random())
    }

    pub fn into_cookie(self, max_age: Duration) -> Cookie<'static> {
        Cookie::build((SESSION_ID, self.base64encoded()))
            .path("/")
            .same_site(SameSite::Strict)
            .max_age(max_age)
            .http_only(true)
            .secure(true)
            .build()
    }

    pub async fn info(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Option<SessionInfo>, sqlx::Error> {
        let session_id_hash = self.hash_sha256();

        sqlx::query_as!(
            SessionInfo,
            r#"
            SELECT user_id, created_at, expires_at, user_agent
            FROM sessions WHERE session_id_hash = ?
            "#,
            session_id_hash
        )
        .fetch_optional(pool)
        .await
    }
}

pub fn expired_session_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_ID, ""))
        .path("/")
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(-3600)) // Expire 1 hour ago
        .http_only(true)
        .secure(true)
        .build()
}

impl Default for SessionId {
    fn default() -> Self {
        Self(Token::random())
    }
}

impl SessionInfo {
    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }

    pub fn validate(self) -> Result<Verified<SessionInfo>, SessionValidationError> {
        self.try_into()
    }
}

impl TryFrom<SessionInfo> for Verified<SessionInfo> {
    type Error = SessionValidationError;

    fn try_from(session_info: SessionInfo) -> Result<Self, Self::Error> {
        if session_info.is_expired() {
            return Err(SessionValidationError::SessionExpired);
        }

        Ok(Verified(session_info))
    }
}

impl Authorizable for Verified<SessionInfo> {
    async fn has_permission(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        let user_id = self.0.user_id;

        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM permissions p
                INNER JOIN user_permissions up ON up.permission_id = p.id
                WHERE up.user_id = ? AND p.permission = ?
            )
            "#,
            user_id,
            permission
        )
        .fetch_one(pool)
        .await?;

        Ok(exists != 0)
    }

    async fn permissions(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        let user_id = self.0.user_id;

        sqlx::query_as!(
            Permission,
            r#"
            SELECT p.id as "id!", p.permission, p.description from permissions p
            INNER JOIN user_permissions up ON up.permission_id = p.id
            WHERE up.user_id = ?
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
    }
}

impl Deref for SessionId {
    type Target = Token<32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl extra::ErrorKind for SessionValidationError {
    fn kind(&self) -> &'static str {
        match self {
            SessionValidationError::SessionExpired => "auth.session.expired",
        }
    }
}

impl IntoResponse for SessionValidationError {
    fn into_response(self) -> Response {
        match self {
            SessionValidationError::SessionExpired => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
        }
    }
}

impl extra::ErrorKind for SessionCookieExtractionError {
    fn kind(&self) -> &'static str {
        match self {
            SessionCookieExtractionError::Base64Decode => "auth.session.cookie.base64-decode",
        }
    }
}

impl IntoResponse for SessionCookieExtractionError {
    fn into_response(self) -> Response {
        match self {
            SessionCookieExtractionError::Base64Decode => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    StatusCode::BAD_REQUEST,
                    Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
        }
    }
}
