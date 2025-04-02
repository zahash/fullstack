use axum::{Json, http::StatusCode};
use axum_macros::debug_handler;
use sysinfo::System;

#[debug_handler]
#[tracing::instrument(ret)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}

#[debug_handler]
#[tracing::instrument(ret)]
pub async fn sysinfo() -> Json<System> {
    Json(System::new_all())
}
