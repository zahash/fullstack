use axum::{
    Json,
    extract::State,
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use axum_macros::debug_handler;
use http::StatusCode;
use serde::Serialize;
use sysinfo::{Disks, System};

use crate::{
    AppState,
    core::{InsufficientPermissionsError, Principal},
};

pub const PATH: &str = "/sysinfo";

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = sysinfo::Info))]
#[derive(Debug, Serialize)]
pub struct Info {
    pub system: SystemInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = sysinfo::SystemInfo))]
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    #[cfg_attr(
        feature = "openapi",
        schema(examples("Ubuntu", "Pixel 9 Pro", "Darwin", "Windows"))
    )]
    pub name: Option<String>,

    #[cfg_attr(feature = "openapi", schema(examples("MyLittleComputer")))]
    pub host_name: Option<String>,

    #[cfg_attr(
        feature = "openapi",
        schema(examples("6.8.0-48-generic", "6.1.84-android14-11", "24.1.0", "20348"))
    )]
    pub kernel_version: Option<String>,

    #[cfg_attr(
        feature = "openapi",
        schema(examples("24.04", "15", "15.1.1", "10 (20348)"))
    )]
    pub os_version: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = sysinfo::MemoryInfo))]
#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    #[cfg_attr(feature = "openapi", schema(examples(16873545728u64)))]
    pub total_memory: u64,

    #[cfg_attr(feature = "openapi", schema(examples(11236237312u64)))]
    pub used_memory: u64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = sysinfo::DiskInfo))]
#[derive(Debug, Serialize)]
pub struct DiskInfo {
    #[cfg_attr(feature = "openapi", schema(examples("nvme0n1p1", "Windows-SSD")))]
    pub name: String,

    #[cfg_attr(feature = "openapi", schema(examples("/mnt/data", r"C:\\")))]
    pub mount_point: String,

    #[cfg_attr(feature = "openapi", schema(examples("ext4", "NTFS")))]
    pub file_system: String,

    #[cfg_attr(feature = "openapi", schema(examples("SSD")))]
    pub kind: String,

    #[cfg_attr(feature = "openapi", schema(examples(1021821579264u64)))]
    pub total_space: u64,

    #[cfg_attr(feature = "openapi", schema(examples(435753508864u64)))]
    pub available_space: u64,
}

impl Default for Info {
    fn default() -> Self {
        let system = {
            let mut system = System::new_all();
            system.refresh_all();
            system
        };
        let disks = Disks::new_with_refreshed_list();

        Self {
            system: SystemInfo {
                name: System::name(),
                host_name: System::host_name(),
                kernel_version: System::kernel_version(),
                os_version: System::os_version(),
            },
            memory: MemoryInfo {
                total_memory: system.total_memory(),
                used_memory: system.used_memory(),
            },
            disks: disks
                .into_iter()
                .map(|disk| DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    file_system: disk.file_system().to_string_lossy().to_string(),
                    kind: disk.kind().to_string(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                })
                .collect(),
        }
    }
}

pub fn method_router() -> MethodRouter<AppState> {
    get(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    operation_id = PATH,
    responses(
        (status = 200, description = "System Information", body = Info),
        (status = 401, description = "Invalid credentials", body = extra::ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = extra::ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "probe"
))]
#[debug_handler]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%principal), skip_all, ret))]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    principal: Principal,
) -> Result<Json<Info>, Error> {
    principal
        .require_permission::<Error>(&pool, "get:/sysinfo")
        .await?;
    Ok(Json(Info::default()))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InsufficientPermissions(#[from] InsufficientPermissionsError),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InsufficientPermissions(err) => err.into_response(),
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
