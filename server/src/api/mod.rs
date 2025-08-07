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
#[openapi(
    paths(
        health::handler,
        login::handler,
        logout::handler,
        signup::handler,
        sysinfo::handler,
        username::check_availability::handler
    ),
    components(schemas(login::Credentials, signup::RequestBody, sysinfo::ResponseBody))
)]
pub struct OpenApiDoc;
