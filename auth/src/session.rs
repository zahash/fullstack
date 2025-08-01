use std::ops::Deref;

use cookie::{Cookie, SameSite, time::Duration};
use dashcache::DashCache;
use data_access::DataAccess;
use http::header::COOKIE;
use tag::Tag;
use time::OffsetDateTime;
use token::Token;

use crate::{Credentials, Permission, Permissions, Verified};

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
    id: i64,
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
        data_access: &DataAccess,
    ) -> Result<Option<SessionInfo>, data_access::Error> {
        let session_id_hash = self.hash_sha256();

        data_access
            .read(
                |pool| {
                    sqlx::query_as!(
                        SessionInfo,
                        r#"
                        SELECT id as "id!", user_id, created_at, expires_at, user_agent
                        FROM sessions WHERE session_id_hash = ?
                        "#,
                        session_id_hash
                    )
                    .fetch_optional(pool)
                },
                "session_info__from__session_id",
                session_id_hash.clone(),
                |value| match value {
                    Some(session_info) => vec![Tag {
                        table: "sessions",
                        primary_key: Some(session_info.id),
                    }],
                    None => vec![Tag {
                        table: "sessions",
                        primary_key: None,
                    }],
                },
                DashCache::new,
            )
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

impl Verified<SessionInfo> {
    pub async fn permissions(
        &self,
        data_access: &DataAccess,
    ) -> Result<Permissions, data_access::Error> {
        let user_id = self.0.user_id;

        data_access
            .read(
                |pool| {
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
                },
                "session_permissions__from__user_id",
                user_id,
                |permissions| {
                    let mut tags = permissions
                        .iter()
                        .map(|p| Tag {
                            table: "permissions",
                            primary_key: Some(p.id),
                        })
                        .collect::<Vec<Tag>>();
                    tags.push(Tag {
                        table: "users",
                        primary_key: Some(user_id),
                    });
                    tags
                },
                DashCache::new,
            )
            .await
            .map(Permissions)
    }
}

impl Deref for SessionId {
    type Target = Token<32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for SessionValidationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SessionValidationError::SessionExpired => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
        }
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for SessionCookieExtractionError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SessionCookieExtractionError::Base64Decode => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
        }
    }
}
