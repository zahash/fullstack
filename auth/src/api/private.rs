use axum::routing::{MethodRouter, get};

use crate::{AppState, core::Principal};

pub const PATH: &str = "/private";

pub fn method_router() -> MethodRouter<AppState> {
    get(handler)
}

#[cfg_attr(feature = "tracing", tracing::instrument(fields(%principal), skip_all, ret))]
pub async fn handler(principal: Principal) -> String {
    let user_id = principal.user_id();

    format!("hello {user_id}")
}
