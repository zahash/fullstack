use std::ops::Deref;

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use cache::{DashCache, Tag};
use time::OffsetDateTime;

use crate::{Base64DecodeError, DataAccess, Permission, Token, Valid, error};

pub struct SessionId(Token<32>);

#[derive(Clone)]
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

    pub async fn info(&self, data_access: &DataAccess) -> Result<Option<SessionInfo>, sqlx::Error> {
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
                    Some(session_info) => vec![Box::new(format!("sessions:{}", session_info.id))],
                    None => vec![Box::new("sessions")],
                },
                DashCache::new,
            )
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
    pub async fn permissions(
        &self,
        data_access: &DataAccess,
    ) -> Result<Vec<Permission>, sqlx::Error> {
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
                        .into_iter()
                        .map(|p| format!("permissions:{}", p.id))
                        .map(|tag| Box::new(tag) as Box<dyn Tag>)
                        .collect::<Vec<Box<dyn Tag + 'static>>>();
                    tags.push(Box::new(format!("users:{}", user_id)));
                    tags
                },
                DashCache::new,
            )
            .await
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
