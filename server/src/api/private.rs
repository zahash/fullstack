use auth::Principal;
use axum::routing::{MethodRouter, get};

use crate::AppState;

pub const PATH: &str = "/private";

pub fn method_router() -> MethodRouter<AppState> {
    get(handler)
}

#[cfg_attr(feature = "tracing", tracing::instrument(fields(%principal), skip_all, ret))]
pub async fn handler(principal: Principal) -> String {
    let user_id = principal.user_id();

    #[cfg(feature = "tracing")]
    tracing::Span::current().record("user_id", tracing::field::display(user_id));

    format!("hello {user_id}")
}
