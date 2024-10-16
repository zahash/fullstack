use anyhow::Context;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use sqlx::{Sqlite, SqlitePool, Type};

use crate::{
    access_token::AccessToken,
    error::{AccessTokenError, AuthError, HandlerError, HandlerErrorKind, SessionError},
    request_id::RequestId,
    session_id::SessionId,
    AppState,
};

#[derive(Debug, PartialEq)]
pub struct UserId(i64);

#[async_trait]
impl FromRequestParts<AppState> for UserId {
    type Rejection = HandlerError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        UserId::from_request_parts(&state.pool, parts)
            .await
            .map_err(|e| HandlerError {
                request_id: RequestId::from(parts),
                kind: e.into(),
            })
    }
}

impl UserId {
    pub async fn from_session_id(
        pool: &SqlitePool,
        session_id: &SessionId,
    ) -> Result<Self, HandlerErrorKind> {
        let session_id_hash = session_id.hash();

        let record = sqlx::query!(
            "SELECT user_id FROM sessions WHERE session_id_hash = ? AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)",
            session_id_hash
        )
        .fetch_optional(pool)
        .await.context("SessionId -> UserId")?
        .ok_or(SessionError::InvalidSessionToken)?;

        Ok(Self(record.user_id))
    }

    pub async fn from_access_token(
        pool: &SqlitePool,
        access_token: &AccessToken,
    ) -> Result<Self, HandlerErrorKind> {
        let access_token_hash = access_token.hash();

        let record = sqlx::query!(
            "SELECT user_id FROM access_tokens WHERE access_token_hash = ? AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)",
            access_token_hash
        )
        .fetch_optional(pool)
        .await.context("AccessToken -> UserId")?
        .ok_or(AccessTokenError::InvalidAccessToken)?;

        Ok(Self(record.user_id))
    }

    pub async fn from_request_parts(
        pool: &SqlitePool,
        parts: &Parts,
    ) -> Result<Self, HandlerErrorKind> {
        let session_id = SessionId::try_from(parts);
        let access_token = AccessToken::try_from(parts);

        match (session_id, access_token) {
            (Ok(session_id), Ok(access_token)) => Err(AuthError::MultipleCredentialsProvided {
                session_id,
                access_token,
            }
            .into()),
            (Ok(session_id), _) => Self::from_session_id(pool, &session_id).await,
            (_, Ok(access_token)) => Self::from_access_token(pool, &access_token).await,
            _ => Err(AuthError::NoCredentialsProvided.into()),
        }
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
