use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use serde::Deserialize;

use crate::{check::username_exists, error::HandlerError, types::Username, AppState};

#[derive(Deserialize)]
pub struct Params {
    pub username: Username,
}

#[debug_handler]
#[tracing::instrument(fields(?username), skip_all, ret)]
pub async fn username_availability(
    State(AppState { pool, .. }): State<AppState>,
    Query(Params { username }): Query<Params>,
) -> Result<StatusCode, HandlerError> {
    match username_exists(&pool, &username)
        .await
        .context("check username availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}
