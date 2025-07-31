use auth::Principal;

pub const PATH: &str = "/private";

#[tracing::instrument(fields(user_id = tracing::field::Empty), skip_all, ret)]
pub async fn handler(principal: Principal) -> String {
    let user_id = principal.user_id();
    tracing::Span::current().record("user_id", tracing::field::display(user_id));
    format!("hello {user_id}")
}
