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
pub const OPEN_API_DOCS_PATH: &str = "/api-docs/openapi.json";

#[cfg(feature = "openapi")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        access_token::generate::handler,
        access_token::verify::handler,
        email::check_availability::handler,
        health::handler,
        login::handler,
        logout::handler,
        signup::handler,
        sysinfo::handler,
        username::check_availability::handler
    ),
    components(schemas(
        access_token::generate::Config,
        login::Credentials,
        signup::RequestBody,
        sysinfo::Info
    ))
)]
struct OpenApiDoc;

#[cfg(all(feature = "openapi", feature = "smtp"))]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        email::check_verification_token::handler,
        email::initiate_verification::handler,
    ),
    components(schemas(email::check_verification_token::RequestBody))
)]
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

// TODO: create permissions management apis
