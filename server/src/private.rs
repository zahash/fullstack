use axum_macros::debug_handler;

use crate::{error::AppError, types::UserId};

#[debug_handler]
#[tracing::instrument(fields(user_id = id), skip_all, ret)]
pub async fn private(UserId(id): UserId) -> Result<String, AppError> {
    tracing::info!("this is private");
    Ok(format!("hello {}", id))
}
