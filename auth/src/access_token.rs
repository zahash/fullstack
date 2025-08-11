use std::ops::Deref;

use dashcache::DashCache;
use data_access::DataAccess;
use tag::Tag;
use time::OffsetDateTime;
use token::Token;

use crate::{Credentials, Permission, Permissions, Verified};

pub struct AccessToken(Token<32>);

impl Credentials for AccessToken {
    type Error = AccessTokenAuthorizationExtractionError;

    fn try_from_headers(headers: &http::HeaderMap) -> Result<Option<Self>, Self::Error> {
        let Some(header_value) = headers.get(http::header::AUTHORIZATION) else {
            return Ok(None);
        };

        let header_value_str = header_value
            .to_str()
            .map_err(|_| AccessTokenAuthorizationExtractionError::NonUTF8HeaderValue)?;

        let Some(token_value) = header_value_str.strip_prefix("Token ") else {
            return Ok(None);
        };

        let token = Token::base64decode(token_value)
            .map_err(|_| AccessTokenAuthorizationExtractionError::Base64Decode)?;

        Ok(Some(AccessToken::from(token)))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AccessTokenAuthorizationExtractionError {
    #[error("Authorization header value must be utf-8")]
    NonUTF8HeaderValue,

    #[error("cannot base64 decode :: Authorization: Token xxx")]
    Base64Decode,
}

#[derive(Debug, Clone)]
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
    ) -> Result<Option<AccessTokenInfo>, data_access::Error> {
        let access_token_hash = self.hash_sha256();

        data_access
            .read(
                |pool| {
                    sqlx::query_as!(
                        AccessTokenInfo,
                        r#"
                        SELECT id as "id!", name, user_id, created_at, expires_at
                        FROM access_tokens
                        WHERE access_token_hash = ?
                        "#,
                        access_token_hash
                    )
                    .fetch_optional(pool)
                },
                "access_token_info__from__access_token_hash",
                access_token_hash.clone(),
                |value| match value {
                    Some(access_token_info) => vec![Tag {
                        table: "access_tokens",
                        primary_key: Some(access_token_info.id),
                    }],
                    None => vec![Tag {
                        table: "access_tokens",
                        primary_key: None,
                    }],
                },
                DashCache::new,
            )
            .await
    }
}

impl Default for AccessToken {
    fn default() -> Self {
        Self(Token::random())
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
    pub async fn permissions(
        &self,
        data_access: &DataAccess,
    ) -> Result<Permissions, data_access::Error> {
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
                        .iter()
                        .map(|p| Tag {
                            table: "permissions",
                            primary_key: Some(p.id),
                        })
                        .collect::<Vec<Tag>>();
                    tags.push(Tag {
                        table: "access_tokens",
                        primary_key: Some(access_token_id),
                    });
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
impl axum::response::IntoResponse for AccessTokenValidationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AccessTokenValidationError::AccessTokenExpired => {
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
impl axum::response::IntoResponse for AccessTokenAuthorizationExtractionError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AccessTokenAuthorizationExtractionError::NonUTF8HeaderValue
            | AccessTokenAuthorizationExtractionError::Base64Decode => {
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
