use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use axum_macros::debug_handler;
use contextual::Context;
use dashcache::DashCache;
use http::StatusCode;
use tag::Tag;
use time::OffsetDateTime;

use crate::{AppState, smtp::VerificationToken};

pub const PATH: &str = "/check/email-verification-token";

#[debug_handler]
pub async fn handler(
    State(AppState { data_access, .. }): State<AppState>,
    Query(token_b64encoded): Query<String>,
) -> Result<StatusCode, Error> {
    let token =
        VerificationToken::base64decode(&token_b64encoded).map_err(|_| Error::Base64decode)?;
    let token_hash = token.hash_sha256();

    #[derive(Debug, Clone)]
    struct Row {
        id: i64,
        user_id: i64,
        expires_at: OffsetDateTime,
    }

    let row = data_access
        .read(
            |pool| {
                sqlx::query_as!(
                    Row,
                    r#"
                    SELECT id as "id!", user_id, expires_at
                    FROM email_verification_tokens
                    WHERE token_hash = ?
                    "#,
                    token_hash
                )
                .fetch_optional(pool)
            },
            "email_verification_token__from__token_hash",
            token_hash.clone(),
            |value| match value {
                Some(row) => vec![Tag {
                    table: "email_verification_tokens",
                    primary_key: Some(row.id),
                }],
                None => vec![Tag {
                    table: "email_verification_tokens",
                    primary_key: None,
                }],
            },
            DashCache::new,
        )
        .await
        .context("select Email VerificationToken")?
        .ok_or(Error::TokenNotFound)?;

    if OffsetDateTime::now_utc() > row.expires_at {
        return Err(Error::TokenExpired);
    }

    data_access
        .write(
            |pool| {
                sqlx::query!(
                    "UPDATE users SET email_verified = 1 WHERE id = ?",
                    row.user_id
                )
                .execute(pool)
            },
            |_| {
                vec![
                    Tag {
                        table: "users",
                        primary_key: Some(row.user_id),
                    },
                    Tag {
                        table: "users",
                        primary_key: None,
                    },
                ]
            },
        )
        .await
        .context("user email_verified")?;

    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cannot base64 decode :: Email VerificationToken")]
    Base64decode,

    #[error("email verification token not found")]
    TokenNotFound,

    #[error("email verification token expired")]
    TokenExpired,

    #[error("{0}")]
    DataAccess(#[from] contextual::Error<data_access::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Base64decode => {
                tracing::info!("{:?}", self);
                (
                    StatusCode::BAD_REQUEST,
                    Json(extra::json_error_response(self)),
                )
                    .into_response()
            }
            Error::TokenNotFound => {
                tracing::info!("{:?}", self);
                (
                    StatusCode::NOT_FOUND,
                    Json(extra::json_error_response(self)),
                )
                    .into_response()
            }
            Error::TokenExpired => {
                tracing::info!("{:?}", self);
                (StatusCode::GONE, Json(extra::json_error_response(self))).into_response()
            }
            Error::DataAccess(_) => {
                tracing::error!("{:?}", self);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
