use anyhow::Context;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use sqlx::SqlitePool;

use crate::{
    error::{AuthError, CookieError, HandlerError, HandlerErrorKind},
    types::{RequestId, UserId},
};

#[async_trait]
impl<S> FromRequestParts<S> for UserId {
    type Rejection = HandlerError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        async fn inner(parts: &mut Parts) -> Result<UserId, HandlerErrorKind> {
            let pool = parts
                .extensions
                .get::<SqlitePool>()
                .context("SqlitePool extension not found")?;

            let jar = CookieJar::from_headers(&parts.headers);

            let session_id = jar
                .get("session_id")
                .ok_or(CookieError::CookieNotFound("session_id"))?
                .value();

            let session = sqlx::query!(
                "SELECT user_id FROM sessions WHERE session_id = ? AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)",
                session_id
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

        inner(parts).await.map_err(|e| HandlerError {
            request_id,
            kind: e.into(),
        })
    }
}
