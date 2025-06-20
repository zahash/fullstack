use std::ops::Deref;

use cache::{DashCache, Tag};
use data_access::DataAccess;
use time::OffsetDateTime;
use token::Token;

use crate::{Permission, Permissions, Verified};

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
pub enum AccessTokenValidationError {
    #[error("access token expired")]
    AccessTokenExpired,
}

impl AccessToken {
    pub fn new() -> Self {
        Self(Token::random())
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

    pub fn verify(self) -> Result<Verified<AccessTokenInfo>, AccessTokenValidationError> {
        self.try_into()
    }
}

impl TryFrom<AccessTokenInfo> for Verified<AccessTokenInfo> {
    type Error = AccessTokenValidationError;

    fn try_from(access_token_info: AccessTokenInfo) -> Result<Self, Self::Error> {
        if access_token_info.is_expired() {
            return Err(AccessTokenValidationError::AccessTokenExpired);
        }

        Ok(Verified(access_token_info))
    }
}

impl Verified<AccessTokenInfo> {
    pub async fn permissions(&self, data_access: &DataAccess) -> Result<Permissions, sqlx::Error> {
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
            .map(Permissions)
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

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for AccessToken {
    fn into_response(self) -> axum::response::Response {
        self.base64encoded().into_response()
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for AccessTokenValidationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AccessTokenValidationError::AccessTokenExpired => {
                error::axum_error_response(axum::http::StatusCode::UNAUTHORIZED, self)
            }
        }
    }
}
