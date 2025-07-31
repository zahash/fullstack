use axum::{
    Json,
    extract::{Query, State},
};
use axum_macros::debug_handler;
use email::Email;
use lettre::transport::smtp::response::Response;

use crate::{AppState, smtp::InitiateEmailVerificationError};

pub const PATH: &str = "/initiate-email-verification";

#[debug_handler]
#[tracing::instrument(fields(?email), skip_all, ret)]
pub async fn handler(
    State(AppState { data_access, smtp }): State<AppState>,
    Query(email): Query<Email>,
) -> Result<Json<Response>, InitiateEmailVerificationError> {
    crate::smtp::initiate_email_verification(&data_access, &smtp, &email)
        .await
        .map(Json)
}
