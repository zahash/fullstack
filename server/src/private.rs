use crate::{error::HandlerError, user_id::UserId};

#[tracing::instrument(fields(%user_id), skip_all, ret)]
pub async fn private(user_id: UserId) -> Result<String, HandlerError> {
    tracing::info!(%user_id);
    Ok(format!("hello {}", user_id))
}
