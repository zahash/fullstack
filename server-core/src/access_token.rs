use std::ops::Deref;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::{Permissions, Principal, Token, UserId, Valid, error};

pub struct AccessToken(Token<32>);

pub struct AccessTokenInfo {
    id: i64,
    pub name: String,
    pub user_id: UserId,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(thiserror::Error, Debug)]
pub enum AccessTokenValiationError {
    #[error("access token expired")]
    AccessTokenExpired,
}

impl AccessToken {
    pub fn new() -> Self {
        Self(Token::new())
    }

    pub async fn info(&self, pool: &SqlitePool) -> Result<Option<AccessTokenInfo>, sqlx::Error> {
        let access_token_hash = self.hash();

        sqlx::query_as!(
            AccessTokenInfo,
            r#"SELECT id as "id!", name, user_id, created_at, expires_at FROM access_tokens WHERE access_token_hash = ?"#,
            access_token_hash
        ).fetch_optional(pool)
        .await
    }
}

impl AccessTokenInfo {
    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }

    pub fn validate(self) -> Result<Valid<AccessTokenInfo>, AccessTokenValiationError> {
        if self.is_expired() {
            return Err(AccessTokenValiationError::AccessTokenExpired);
        }

        Ok(Valid(self))
    }
}

impl Valid<AccessTokenInfo> {
    pub async fn permissions(self, pool: &SqlitePool) -> Result<Permissions, sqlx::Error> {
        let access_token_id = &self.0.id;

        let permissions = sqlx::query_scalar!(
            "SELECT p.permission from permissions p
             INNER JOIN access_token_permissions atp ON atp.permission_id = p.id
             WHERE atp.access_token_id = ?",
            access_token_id
        )
        .fetch_all(pool)
        .await?;

        Ok(Permissions {
            permissions,
            principal: Principal::AccessToken(self.0),
        })
    }
}

impl Deref for AccessToken {
    type Target = Token<32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Token<32>> for AccessToken {
    fn from(value: Token<32>) -> Self {
        Self(value)
    }
}

impl IntoResponse for AccessToken {
    fn into_response(self) -> Response {
        self.base64encoded().into_response()
    }
}

impl IntoResponse for AccessTokenValiationError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenValiationError::AccessTokenExpired => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
        }
    }
}
