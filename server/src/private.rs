use axum_macros::debug_handler;

use crate::{error::AppError, types::UserId};

#[debug_handler]
pub async fn private(UserId(id): UserId) -> Result<String, AppError> {
    Ok(format!("hello {}", id))
}
