use std::ops::Deref;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use time::OffsetDateTime;
use token::Token;

use crate::core::{Credentials, Permission, Verified, permission::Authorizable};

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
    pub id: i64,
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
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Option<AccessTokenInfo>, sqlx::Error> {
        let access_token_hash = self.hash_sha256();

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

impl Authorizable for Verified<AccessTokenInfo> {
    async fn has_permission(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        let access_token_id = self.0.id;

        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM permissions p
                INNER JOIN access_token_permissions atp ON atp.permission_id = p.id
                WHERE atp.access_token_id = ? AND p.permission = ?
            )
            "#,
            access_token_id,
            permission
        )
        .fetch_one(pool)
        .await?;

        Ok(exists != 0)
    }

    async fn permissions(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        let access_token_id = self.0.id;

        sqlx::query_as!(
            Permission,
            r#"
            SELECT p.id as "id!", p.permission, p.description from permissions p
            INNER JOIN access_token_permissions atp ON atp.permission_id = p.id
            WHERE atp.access_token_id = ?
            "#,
            access_token_id
        )
        .fetch_all(pool)
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

impl extra::ErrorKind for AccessTokenValidationError {
    fn kind(&self) -> &'static str {
        match self {
            AccessTokenValidationError::AccessTokenExpired => "auth.access-token.expired",
        }
    }
}

impl IntoResponse for AccessTokenValidationError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenValidationError::AccessTokenExpired => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
        }
    }
}

impl extra::ErrorKind for AccessTokenAuthorizationExtractionError {
    fn kind(&self) -> &'static str {
        match self {
            AccessTokenAuthorizationExtractionError::NonUTF8HeaderValue => {
                "auth.access-token.authorization-header.non-utf8"
            }
            AccessTokenAuthorizationExtractionError::Base64Decode => {
                "auth.access-token.authorization-header.base64-decode"
            }
        }
    }
}

impl IntoResponse for AccessTokenAuthorizationExtractionError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenAuthorizationExtractionError::NonUTF8HeaderValue
            | AccessTokenAuthorizationExtractionError::Base64Decode => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);
                (
                    StatusCode::BAD_REQUEST,
                    Json(extra::ErrorResponse::from(self)),
                )
                    .into_response()
            }
        }
    }
}
