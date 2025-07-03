use crate::principal::Principal;

#[tracing::instrument(fields(user_id = tracing::field::Empty), skip_all, ret)]
pub async fn private(principal: Principal) -> String {
    let user_id = principal.user_id();
    tracing::Span::current().record("user_id", tracing::field::debug(user_id));
    format!("hello {:?}", user_id)
}
