use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{check::username_exists, error::InternalError, types::Username, AppState};

#[derive(Deserialize)]
pub struct Params {
    pub username: Username,
}

#[tracing::instrument(fields(?username), skip_all, ret)]
pub async fn username_availability<T>(
    State(AppState { pool, .. }): State<AppState<T>>,
    Query(Params { username }): Query<Params>,
) -> Result<StatusCode, InternalError> {
    match username_exists(&pool, &username)
        .await
        .context("check username availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}
