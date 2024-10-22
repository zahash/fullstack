use crate::{error::HandlerError, types::UserId};

#[tracing::instrument(fields(?user_id), skip_all, ret)]
pub async fn private(user_id: UserId) -> Result<String, HandlerError> {
    Ok(format!("hello {:?}", user_id))
}
