use crate::types::{InsufficientPermissions, Permissions};

#[tracing::instrument(fields(user_id = tracing::field::Empty), skip_all, ret)]
pub async fn private(permissions: Permissions) -> Result<String, InsufficientPermissions> {
    let user_id = permissions.user_id();
    tracing::Span::current().record("user_id", &tracing::field::debug(user_id));

    Ok(format!("hello {:?}", user_id))
}

// TODO: avoid multiple redundant sql queries when multiple extractors are used
// eg: pub async fn private(user_id: UserId, auth: Auth, permissions: Permissions) -> String;
// same database query is run to get each extractor. avoid this using lazy evaluation / caching
