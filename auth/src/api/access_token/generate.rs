use std::time::Duration;

use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{MethodRouter, post},
};
use axum_macros::debug_handler;
use contextual::Context;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    AppState,
    core::{AccessToken, InsufficientPermissionsError, Principal},
};

pub const PATH: &str = "/access-token/generate";

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = access_token::generate::Config))]
#[derive(Deserialize, Debug)]
pub struct Config {
    #[cfg_attr(feature = "openapi", schema(example = "my-token"))]
    name: String,

    #[cfg_attr(feature = "openapi", schema(example = 3600u64, value_type = u64))]
    ttl_sec: Option<u64>,
}

pub fn method_router() -> MethodRouter<AppState> {
    post(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = PATH,
    operation_id = PATH,
    request_body(
        content = Config,
        content_type = "application/x-www-form-urlencoded",
    ),
    responses(
        (status = 200, description = "Access token generated successfully", body = String),
        (status = 403, description = "Insufficient permissions", body = extra::ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "access_token"
))]
#[debug_handler]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%principal, ?settings), skip_all))]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    principal: Principal,
    Form(settings): Form<Config>,
) -> Result<(StatusCode, String), Error> {
    principal
        .require_permission::<Error>(&pool, "post:/access-token/generate")
        .await?;

    let user_id = principal.user_id();

    let access_token = AccessToken::new();
    let access_token_hash = access_token.hash_sha256();
    let created_at = OffsetDateTime::now_utc();
    let expires_at = settings
        .ttl_sec
        .map(|sec| created_at + Duration::from_secs(sec));

    sqlx::query!(
        r#"
        INSERT INTO access_tokens
        (name, access_token_hash, user_id, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
        settings.name,
        access_token_hash,
        user_id,
        created_at,
        expires_at,
    )
    .execute(&pool)
    .await
    .context("insert access token")?;

    #[cfg(feature = "tracing")]
    tracing::info!(?expires_at, "access_token created");

    Ok((StatusCode::CREATED, access_token.base64encoded()))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InsufficientPermissions(#[from] InsufficientPermissionsError),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InsufficientPermissions(err) => err.into_response(),
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
