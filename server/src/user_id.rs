use std::fmt::Display;

use anyhow::Context;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use sqlx::{Sqlite, SqlitePool, Type};

use crate::{
    error::{AuthError, HandlerErrorKind, HandlerError},
    request_id::RequestId,
    session_id::SessionId,
    AppState,
};

pub struct UserId(i64);

#[async_trait]
impl FromRequestParts<AppState> for UserId {
    type Rejection = HandlerError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        async fn inner(parts: &mut Parts, pool: &SqlitePool) -> Result<UserId, HandlerErrorKind> {
            let jar = CookieJar::from_headers(&parts.headers);
            let session_id = SessionId::try_from(&jar)?;
            let session_id_hash = session_id.hash();

            let session = sqlx::query!(
                "SELECT user_id FROM sessions WHERE session_id_hash = ? AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)",
                session_id_hash
            )
            .fetch_optional(pool)
            .await.context("extractor: session_id -> UserId")?
            .ok_or(AuthError::InvalidSession)?;

            Ok(UserId(session.user_id))
        }

        let request_id = parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(|| {
                tracing::warn!("unable to get RequestId extension when extracting UserId");
                RequestId::unknown()
            });

        inner(parts, &state.pool).await.map_err(|e| HandlerError {
            request_id,
            kind: e.into(),
        })
    }
}

impl Type<Sqlite> for UserId {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, Sqlite> for UserId {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as sqlx::Encode<Sqlite>>::encode_by_ref(&self.0, buf)
    }
}

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        UserId(value)
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserId({})", self.0)
    }
}
