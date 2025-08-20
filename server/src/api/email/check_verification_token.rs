use std::str::FromStr;

use axum::{Form, Json, extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use contextual::Context;

use email::Email;
use extra::ErrorResponse;
use http::StatusCode;
use serde::Deserialize;

use time::OffsetDateTime;

use crate::{AppState, smtp::VerificationToken};

pub const PATH: &str = "/check/email-verification-token";

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = email::check_verification_token::RequestBody))]
#[derive(Deserialize)]
pub struct RequestBody {
    #[cfg_attr(feature = "openapi", schema(example = "joe@smith.com"))]
    pub email: String,

    #[cfg_attr(feature = "openapi", schema(example = "gZwnqQ"))]
    pub token_b64encoded: String,
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = PATH,
    operation_id = PATH,
    request_body(
        content = RequestBody,
        content_type = "application/x-www-form-urlencoded",
    ),
    responses(
        (status = 200, description = "Email verified successfully"),
        (status = 400, description = "Invalid or malformed token", body = ErrorResponse),
        (status = 404, description = "Token not found", body = ErrorResponse),
        (status = 410, description = "Token expired", body = ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "check"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%email), skip_all, ret))]
#[debug_handler]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    Form(RequestBody {
        email,
        token_b64encoded,
    }): Form<RequestBody>,
) -> Result<StatusCode, Error> {
    let email = Email::from_str(&email).map_err(Error::InvalidEmail)?;

    let token_hash = VerificationToken::base64decode(&token_b64encoded)
        .map_err(|_| Error::Base64decode)?
        .hash_sha256();

    let record = sqlx::query!(
        r#"
        SELECT token_hash, expires_at
        FROM email_verification_tokens
        WHERE email = ?
        "#,
        email
    )
    .fetch_optional(&pool)
    .await
    .context("select Email VerificationToken")?
    .ok_or(Error::VerificationNotInitialized(email.clone()))?;

    // delete the token row unconditionally
    // this ensures the token is used exactly once
    // and deleted regardless of subsequent checks/outcomes
    sqlx::query!(
        "DELETE FROM email_verification_tokens WHERE email = ?",
        email
    )
    .execute(&pool)
    .await
    .context("delete Email VerificationToken")?;

    if token_hash != record.token_hash {
        return Err(Error::TokenMismatch);
    }

    if OffsetDateTime::now_utc() > record.expires_at {
        return Err(Error::TokenExpired);
    }

    sqlx::query!("UPDATE users SET email_verified = 1 WHERE email = ?", email)
        .execute(&pool)
        .await
        .context("user email_verified")?;

    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidEmail(&'static str),

    #[error("verification not initialized for email `{0}`")]
    VerificationNotInitialized(Email),

    #[error("cannot base64 decode :: Email VerificationToken")]
    Base64decode,

    #[error("email verification token mismatch")]
    TokenMismatch,

    #[error("email verification token expired")]
    TokenExpired,

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::InvalidEmail(_) => "email.invalid",
            Error::VerificationNotInitialized(_) => "email.verification.not-initialized",
            Error::Base64decode => "email.verification.token.base64-decode",
            Error::TokenMismatch => "email.verification.token.mismatch",
            Error::TokenExpired => "email.verification.token.expired",
            Error::Sqlx(_) => "email.verification.sqlx",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InvalidEmail(_) | Error::Base64decode | Error::TokenMismatch => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(self))).into_response()
            }
            Error::VerificationNotInitialized(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::NOT_FOUND, Json(ErrorResponse::from(self))).into_response()
            }
            Error::TokenExpired => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::GONE, Json(ErrorResponse::from(self))).into_response()
            }
            Error::Sqlx(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
