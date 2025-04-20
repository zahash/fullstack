use crate::types::{InsufficientPermissionsError, Permissions};

#[tracing::instrument(fields(user_id = tracing::field::Empty), skip_all, ret)]
pub async fn private(permissions: Permissions) -> Result<String, InsufficientPermissionsError> {
    let user_id = permissions.user_id();
    tracing::Span::current().record("user_id", &tracing::field::debug(user_id));

    Ok(format!("hello {:?}", user_id))
}
