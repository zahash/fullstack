use std::time::Duration;

use axum::{
    Form,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use time::OffsetDateTime;

use server_core::{
    AccessToken, AccessTokenValiationError, AppState, AuthorizationHeader,
    AuthorizationHeaderError, Context, InsufficientPermissionsError, InternalError, Permissions,
    error,
};

#[derive(Deserialize, Debug)]
pub struct AccessTokenSettings {
    name: String,
    ttl: Option<Duration>,
}

#[debug_handler]
#[tracing::instrument(fields(user_id = tracing::field::Empty, ?settings), skip_all)]
pub async fn generate_access_token(
    State(AppState { pool, .. }): State<AppState>,
    permissions: Permissions,
    Form(settings): Form<AccessTokenSettings>,
) -> Result<(StatusCode, AccessToken), AccessTokenGenerationError> {
    permissions.require("access_token:create")?;

    let user_id = permissions.user_id();
    tracing::Span::current().record("user_id", &tracing::field::debug(user_id));

    let access_token = AccessToken::new();
    let access_token_hash = access_token.hash();
    let created_at = OffsetDateTime::now_utc();
    let expires_at = settings.ttl.map(|ttl| created_at + ttl);

    sqlx::query!(
            "INSERT INTO access_tokens (name, access_token_hash, user_id, created_at, expires_at) VALUES (?, ?, ?, ?, ?)",
            settings.name,
            access_token_hash,
            user_id,
            created_at,
            expires_at,
        )
        .execute(&pool)
        .await.context("insert access_token")?;

    tracing::info!(?expires_at, "access_token created");

    Ok((StatusCode::CREATED, access_token))
}

#[debug_handler]
pub async fn check_access_token(
    State(AppState { pool, .. }): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, CheckAccessTokenError> {
    let Some(AuthorizationHeader::AccessToken(access_token)) =
        AuthorizationHeader::try_from_headers(&headers)?
    else {
        return Err(CheckAccessTokenError::AccessTokenHeaderNotFound);
    };

    let info = access_token
        .info(&pool)
        .await
        .context("AccessToken -> AccessTokenInfo")?
        .ok_or(CheckAccessTokenError::UnAssociatedAccessToken)?;

    info.validate()?;

    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum AccessTokenGenerationError {
    #[error("{0}")]
    Permission(#[from] InsufficientPermissionsError),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[derive(thiserror::Error, Debug)]
pub enum CheckAccessTokenError {
    #[error("{0}")]
    AuthorizationHeader(#[from] AuthorizationHeaderError),

    #[error(
        "access token not found in header. expected `Authorization: Token <your-access-token>`"
    )]
    AccessTokenHeaderNotFound,

    #[error("access token not associated with any account")]
    UnAssociatedAccessToken,

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValiationError),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

impl IntoResponse for AccessTokenGenerationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AccessTokenGenerationError::Permission(err) => err.into_response(),
            AccessTokenGenerationError::Internal(err) => err.into_response(),
        }
    }
}

impl IntoResponse for CheckAccessTokenError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CheckAccessTokenError::AuthorizationHeader(err) => err.into_response(),
            CheckAccessTokenError::AccessTokenHeaderNotFound
            | CheckAccessTokenError::UnAssociatedAccessToken => {
                tracing::info!("{:?}", self);
                (StatusCode::UNAUTHORIZED, error(&self.to_string())).into_response()
            }
            CheckAccessTokenError::AccessTokenValidation(err) => err.into_response(),
            CheckAccessTokenError::Internal(err) => err.into_response(),
        }
    }
}
