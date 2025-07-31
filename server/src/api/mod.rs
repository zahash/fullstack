pub mod access_token;
pub mod email;
pub mod health;
pub mod login;
pub mod logout;
pub mod private;
pub mod signup;
pub mod sysinfo;
pub mod username;

#[cfg(feature = "openapi")]
#[derive(utoipa::OpenApi)]
#[openapi(paths(signup::handler), components(schemas(signup::RequestBody)))]
pub struct OpenApiDoc;
