use axum::Json;
use axum_macros::debug_handler;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};

#[debug_handler]
#[tracing::instrument(ret)]
pub async fn health() -> Json<System> {
    Json(System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(ProcessRefreshKind::everything()),
    ))
}
