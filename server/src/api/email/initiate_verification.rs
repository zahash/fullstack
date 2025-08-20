use std::str::FromStr;

use axum::{Form, Json, extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use email::Email;
use extra::ErrorResponse;
use http::StatusCode;
use serde::Deserialize;

use crate::{AppState, smtp::InitiateEmailVerificationError};

pub const PATH: &str = "/initiate-email-verification";

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = email::initiate_verification::RequestBody))]
#[derive(Deserialize)]
pub struct RequestBody {
    #[cfg_attr(feature = "openapi", schema(examples("joe@smith.com")))]
    pub email: String,
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = PATH,
    operation_id = PATH,
    request_body(
        content = RequestBody,
        content_type = "application/x-www-form-urlencoded",
    ),
    responses(
        (status = 200, description = "Verification email sent successfully"),
        (status = 400, description = "Invalid email address or request"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "email"
))]
#[debug_handler]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%email), skip_all, ret))]
pub async fn handler(
    State(AppState { pool, smtp }): State<AppState>,
    Form(RequestBody { email }): Form<RequestBody>,
) -> Result<StatusCode, Error> {
    let email = Email::from_str(&email).map_err(Error::InvalidEmail)?;

    match crate::smtp::initiate_email_verification(&pool, &smtp, &email).await? {
        None => {
            #[cfg(feature = "tracing")]
            tracing::info!("initiate_email_verification no-op");

            Ok(StatusCode::OK)
        }
        Some(response) => match response.is_positive() {
            true => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", response);

                Ok(StatusCode::OK)
            }
            false => {
                #[cfg(feature = "tracing")]
                tracing::warn!("{:?}", response);

                Ok(StatusCode::SERVICE_UNAVAILABLE)
            }
        },
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidEmail(&'static str),

    #[error("{0}")]
    InitiateEmailVerification(#[from] InitiateEmailVerificationError),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::InvalidEmail(_) => "email.invalid",
            Error::InitiateEmailVerification(err) => err.kind(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InvalidEmail(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(self))).into_response()
            }
            Error::InitiateEmailVerification(err) => err.into_response(),
        }
    }
}
