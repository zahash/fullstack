pub mod access_token;
pub mod email;
pub mod heartbeat;
pub mod introspect;
pub mod key_rotation;
pub mod login;
pub mod logout;
pub mod permissions;
pub mod private;
pub mod signup;
pub mod sysinfo;
pub mod username;

#[cfg(feature = "openapi")]
pub const OPEN_API_DOCS_PATH: &str = "/api-docs/openapi.json";

#[cfg(feature = "openapi")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        access_token::generate::handler,
        access_token::verify::handler,
        email::check_availability::handler,
        heartbeat::handler,
        key_rotation::handler,
        login::handler,
        logout::handler,
        permissions::handler,
        permissions::assign::handler,
        signup::handler,
        sysinfo::handler,
        username::check_availability::handler
    ),
    components(schemas(
        access_token::generate::Config,
        crate::core::Permission,
        key_rotation::RequestBody,
        login::Credentials,
        permissions::assign::RequestBody,
        signup::RequestBody,
        sysinfo::Info
    ))
)]
struct OpenApiDoc;

#[cfg(all(feature = "openapi", feature = "smtp"))]
#[derive(utoipa::OpenApi)]
#[openapi(paths(email::verify_email::handler, email::initiate_verification::handler,))]
struct SmtpOpenApiDoc;

#[cfg(feature = "openapi")]
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;

    let mut openapi = utoipa::openapi::OpenApi::default();
    openapi.merge(OpenApiDoc::openapi());

    #[cfg(feature = "smtp")]
    openapi.merge(SmtpOpenApiDoc::openapi());

    openapi
}
