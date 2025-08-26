use auth::Principal;

pub const PATH: &str = "/private";

#[cfg_attr(feature = "tracing", tracing::instrument(fields(%principal), skip_all, ret))]
pub async fn handler(principal: Principal) -> String {
    let user_id = principal.user_id();

    #[cfg(feature = "tracing")]
    tracing::Span::current().record("user_id", tracing::field::display(user_id));

    format!("hello {user_id}")
}
