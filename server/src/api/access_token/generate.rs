use std::time::Duration;

use auth::{AccessToken, InsufficientPermissionsError, Principal};
use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use axum_macros::debug_handler;
use contextual::Context;
use serde::Deserialize;
use tag::Tag;
use time::OffsetDateTime;

use crate::AppState;

pub const PATH: &str = "/access-token/generate";

#[derive(Deserialize, Debug)]
pub struct AccessTokenSettings {
    name: String,
    ttl: Option<Duration>,
}

#[debug_handler]
#[tracing::instrument(fields(user_id = tracing::field::Empty, ?settings), skip_all)]
pub async fn handler(
    State(AppState { data_access, .. }): State<AppState>,
    principal: Principal,
    Form(settings): Form<AccessTokenSettings>,
) -> Result<(StatusCode, AccessToken), Error> {
    let permissions = principal
        .permissions(&data_access)
        .await
        .context("get permissions")?;

    permissions.require("access_token:create")?;

    let user_id = principal.user_id();
    tracing::Span::current().record("user_id", tracing::field::display(user_id));

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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Permission(#[from] InsufficientPermissionsError),

    #[error("{0}")]
    DataAccess(#[from] contextual::Error<data_access::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Permission(err) => err.into_response(),
            Error::DataAccess(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
