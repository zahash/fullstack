use axum::{
    Json,
    extract::{Query, State},
};
use axum_macros::debug_handler;
use email::Email;
use lettre::transport::smtp::response::Response;

use crate::{AppState, smtp::InitiateEmailVerificationError};

pub const PATH: &str = "/initiate-email-verification";

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    operation_id = PATH,
    params(
        ("email" = String, Query, description = "Email address to initiate verification for", example = "joe@smith.com")
    ),
    responses(
        (status = 200, description = "Verification email sent successfully"),
        (status = 400, description = "Invalid email address or request"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "email"
))]
#[debug_handler]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(?email), skip_all, ret))]
pub async fn handler(
    State(AppState { data_access, smtp }): State<AppState>,
    Query(email): Query<Email>,
) -> Result<Json<Response>, InitiateEmailVerificationError> {
    // TODO: malicious requests could be sent that initiates email verification
    // for random emails
    // maybe require auth for this

    crate::smtp::initiate_email_verification(&data_access, &smtp, &email)
        .await
        .map(Json)
}
