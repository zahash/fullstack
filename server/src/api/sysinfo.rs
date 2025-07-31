use axum::Json;
use axum_macros::debug_handler;
use sysinfo::System;

pub const PATH: &str = "/sysinfo";

#[debug_handler]
#[tracing::instrument(ret)]
pub async fn handler() -> Json<System> {
    Json(System::new_all())
}
