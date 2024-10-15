use crate::{error::HandlerError, user_id::UserId};

// #[tracing::instrument(fields(user_id = user_id), skip_all, ret)]
pub async fn private(user_id: UserId) -> Result<String, HandlerError> {
    tracing::info!("this is private");
    Ok(format!("hello {}", user_id))
}
