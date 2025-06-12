use std::ops::Deref;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cache::{DashCache, Tag};
use time::OffsetDateTime;

use crate::{DataAccess, Permission, Token, Valid, error};

pub struct AccessToken(Token<32>);

#[derive(Clone)]
pub struct AccessTokenInfo {
    id: i64,
    pub name: String,
    pub user_id: i64,
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

    pub async fn info(
        &self,
        data_access: &DataAccess,
    ) -> Result<Option<AccessTokenInfo>, sqlx::Error> {
        let access_token_hash = self.hash_sha256();

        data_access.read(
            |pool| sqlx::query_as!(
                    AccessTokenInfo,
                    r#"SELECT id as "id!", name, user_id, created_at, expires_at FROM access_tokens WHERE access_token_hash = ?"#,
                    access_token_hash
                ).fetch_optional(pool),
            "access_token_info__from__access_token_hash",
            access_token_hash.clone(),
            |value| {
                match value {
                    Some(access_token_info) => vec![Box::new(format!("access_tokens:{}", access_token_info.id))],
                    None => vec![Box::new("access_tokens")],
                }
            },
            DashCache::new
        ).await
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
    pub async fn permissions(
        &self,
        data_access: &DataAccess,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        let access_token_id = self.0.id;

        data_access
            .read(
                |pool| {
                    sqlx::query_as!(
                        Permission,
                        r#"SELECT p.id as "id!", p.permission, p.description from permissions p
                        INNER JOIN access_token_permissions atp ON atp.permission_id = p.id
                        WHERE atp.access_token_id = ?"#,
                        access_token_id
                    )
                    .fetch_all(pool)
                },
                "access_token_permissions__from__access_token_id",
                access_token_id,
                |permissions| {
                    let mut tags = permissions
                        .into_iter()
                        .map(|p| format!("permissions:{}", p.id))
                        .map(|tag| Box::new(tag) as Box<dyn Tag>)
                        .collect::<Vec<Box<dyn Tag + 'static>>>();
                    tags.push(Box::new(format!("access_tokens:{}", access_token_id)));
                    tags
                },
                DashCache::new,
            )
            .await
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
