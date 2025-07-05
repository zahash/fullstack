use std::time::Duration;

use auth::{
    AccessToken, AccessTokenAuthorizationExtractionError, AccessTokenValidationError, Credentials,
    InsufficientPermissionsError, Principal,
};
use axum::{
    Form, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_macros::debug_handler;
use boxer::{Boxer, Context};
use extra::json_error_response;
use serde::Deserialize;
use tag::Tag;
use time::OffsetDateTime;

use crate::AppState;

#[derive(Deserialize, Debug)]
pub struct AccessTokenSettings {
    name: String,
    ttl: Option<Duration>,
}

#[debug_handler]
#[tracing::instrument(fields(user_id = tracing::field::Empty, ?settings), skip_all)]
pub async fn generate_access_token(
    State(AppState { data_access, .. }): State<AppState>,
    principal: Principal,
    Form(settings): Form<AccessTokenSettings>,
) -> Result<(StatusCode, AccessToken), AccessTokenGenerationError> {
    let permissions = principal
        .permissions(&data_access)
        .await
        .context("get permissions")?;

    permissions.require("access_token:create")?;

    let user_id = principal.user_id();
    tracing::Span::current().record("user_id", tracing::field::debug(user_id));

    let access_token = AccessToken::new();
    let access_token_hash = access_token.hash_sha256();
    let created_at = OffsetDateTime::now_utc();
    let expires_at = settings.ttl.map(|ttl| created_at + ttl);

    data_access
        .write(
            |pool| {
                sqlx::query!(
                    r#"
                    INSERT INTO access_tokens
                    (name, access_token_hash, user_id, created_at, expires_at)
                    VALUES (?, ?, ?, ?, ?)
                    RETURNING id as "id!"
                    "#,
                    settings.name,
                    access_token_hash,
                    user_id,
                    created_at,
                    expires_at,
                )
                .fetch_one(pool)
            },
            |value| {
                vec![
                    Tag {
                        table: "access_tokens",
                        primary_key: None,
                    },
                    Tag {
                        table: "access_tokens",
                        primary_key: Some(value.id),
                    },
                ]
            },
        )
        .await
        .context("insert access token")?;

    tracing::info!(?expires_at, "access_token created");

    Ok((StatusCode::CREATED, access_token))
}

#[debug_handler]
#[tracing::instrument(skip_all, ret)]
pub async fn check_access_token(
    State(AppState { data_access, .. }): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, CheckAccessTokenError> {
    let access_token = AccessToken::try_from_headers(&headers)?
        .ok_or_else(|| CheckAccessTokenError::AccessTokenHeaderNotFound)?;

    let info = access_token
        .info(&data_access)
        .await
        .context("AccessToken -> AccessTokenInfo")?
        .ok_or(CheckAccessTokenError::UnAssociatedAccessToken)?;

    tracing::info!(
        "user id = {}; access token name = {}",
        info.user_id,
        info.name
    );

    info.verify()?;

    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum AccessTokenGenerationError {
    #[error("{0}")]
    Permission(#[from] InsufficientPermissionsError),

    #[error("{0:?}")]
    Internal(#[from] Boxer),
}

#[derive(thiserror::Error, Debug)]
pub enum CheckAccessTokenError {
    #[error("{0}")]
    AccessTokenAuthorizationExtractionError(#[from] AccessTokenAuthorizationExtractionError),

    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenHeaderNotFound,

    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValidationError),

    #[error("{0:?}")]
    Internal(#[from] Boxer),
}

impl IntoResponse for AccessTokenGenerationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AccessTokenGenerationError::Permission(err) => err.into_response(),
            AccessTokenGenerationError::Internal(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl IntoResponse for CheckAccessTokenError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CheckAccessTokenError::AccessTokenAuthorizationExtractionError(err) => {
                err.into_response()
            }
            CheckAccessTokenError::AccessTokenHeaderNotFound
            | CheckAccessTokenError::UnAssociatedAccessToken => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, Json(json_error_response(self))).into_response()
            }
            CheckAccessTokenError::AccessTokenValidation(err) => err.into_response(),
            CheckAccessTokenError::Internal(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
