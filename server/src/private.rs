use crate::{error::HandlerError, user_id::UserId};

#[tracing::instrument(fields(user_id = id), skip_all, ret)]
pub async fn private(UserId(id): UserId) -> Result<String, HandlerError> {
    tracing::info!("this is private");
    Ok(format!("hello {}", id))
}
