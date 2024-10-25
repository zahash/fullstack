use crate::types::UserId;

#[tracing::instrument(fields(?user_id), skip_all, ret)]
pub async fn private(user_id: UserId) -> String {
    format!("hello {:?}", user_id)
}
