use anyhow::Context;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use sqlx::SqlitePool;

use crate::{
    error::{HandlerError, AuthError, CookieError, RequestIdCtx},
    types::{RequestId, UserId},
};

#[async_trait]
impl<S> FromRequestParts<S> for UserId {
    type Rejection = HandlerError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(|| RequestId::unknown());

        let pool = parts
            .extensions
            .get::<SqlitePool>()
            .context("SqlitePool extension not found")
            .request_id(request_id.clone())?;

        let jar = CookieJar::from_headers(&parts.headers);

        let session_id = jar
            .get("session_id")
            .ok_or(CookieError::CookieNotFound("session_id"))
            .request_id(request_id.clone())?
            .value();

        let session = sqlx::query!(
                "SELECT user_id FROM sessions WHERE session_id = ? AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)",
                session_id
            )
            .fetch_optional(pool)
            .await.context("extractor: session_id -> UserId").request_id(request_id.clone())?
        .ok_or(AuthError::InvalidSession).request_id(request_id)?;

        Ok(UserId(session.user_id))
    }
}
