use std::ops::Deref;

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::{
    error::{Context, InternalError, error, security_error},
    token::Token,
    types::{Permissions, Principal, UserId, Valid},
};

pub struct AccessToken(Token<32>);

pub struct AccessTokenInfo {
    id: i64,
    pub name: String,
    pub user_id: UserId,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AccessTokenExtractionError {
    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenHeaderNotFound,

    #[error("invalid access token format. must be in the form 'Token <your-access-token>'")]
    InvalidAccessTokenFormat,

    #[error("malformed access token")]
    MalformedAccessToken,
}

#[derive(thiserror::Error, Debug)]
pub enum AccessTokenInfoError {
    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

    #[error("{0:?}")]
    Internal(#[from] InternalError),
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

    pub async fn info(&self, pool: &SqlitePool) -> Result<AccessTokenInfo, AccessTokenInfoError> {
        let access_token_hash = self.hash();

        sqlx::query_as!(
            AccessTokenInfo,
            r#"SELECT id as "id!", name, user_id, created_at, expires_at FROM access_tokens WHERE access_token_hash = ?"#,
            access_token_hash
        ).fetch_optional(pool)
        .await
        .context("AccessToken -> AccessTokenInfo")?
        .ok_or(AccessTokenInfoError::UnAssociatedAccessToken)
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

impl IntoResponse for AccessToken {
    fn into_response(self) -> Response {
        self.base64encoded().into_response()
    }
}

impl TryFrom<&HeaderMap> for AccessToken {
    type Error = AccessTokenExtractionError;

    fn try_from(headers: &HeaderMap) -> Result<Self, Self::Error> {
        let header_value = headers
            .get("Authorization")
            .ok_or(AccessTokenExtractionError::AccessTokenHeaderNotFound)?;

        let token_str = header_value
            .to_str()
            .ok()
            .and_then(|s| s.strip_prefix("Token "))
            .ok_or(AccessTokenExtractionError::InvalidAccessTokenFormat)?;

        Token::base64decode(token_str)
            .map(|token| AccessToken(token))
            .map_err(|_| AccessTokenExtractionError::MalformedAccessToken)
    }
}

impl IntoResponse for AccessTokenExtractionError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenExtractionError::AccessTokenHeaderNotFound
            | AccessTokenExtractionError::InvalidAccessTokenFormat => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            AccessTokenExtractionError::MalformedAccessToken => {
                tracing::error!("!SECURITY! {:?}", self);
                (StatusCode::UNAUTHORIZED, security_error(&self.to_string())).into_response()
            }
        }
    }
}

impl IntoResponse for AccessTokenInfoError {
    fn into_response(self) -> Response {
        match self {
            AccessTokenInfoError::UnAssociatedAccessToken => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            AccessTokenInfoError::Internal(err) => err.into_response(),
        }
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
