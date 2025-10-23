use std::str::FromStr;

use axum::{
    Form, Json,
    extract::State,
    response::IntoResponse,
    routing::{MethodRouter, post},
};
use axum_extra::extract::Host;
use axum_macros::debug_handler;
use contextual::Context;
use email::Email;
use extra::ErrorResponse;
use http::StatusCode;
use serde::Deserialize;

use super::{
    SendVerificationEmailError, send_verification_email, verification_link, verification_token,
};
use crate::AppState;

pub const PATH: &str = "/initiate-email-verification";

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = email::initiate_verification::RequestBody))]
#[derive(Deserialize)]
pub struct RequestBody {
    #[cfg_attr(feature = "openapi", schema(examples("joe@smith.com")))]
    pub email: String,
}

pub fn method_router() -> MethodRouter<AppState> {
    post(handler)
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
    State(AppState {
        pool,
        smtp,
        secrets,
        ..
    }): State<AppState>,
    Host(host): Host,
    Form(RequestBody { email }): Form<RequestBody>,
) -> Result<StatusCode, Error> {
    let email = Email::from_str(&email).map_err(Error::InvalidEmailFormat)?;

    let record = sqlx::query!(
        r#"SELECT email_verified FROM users WHERE email = ? LIMIT 1"#,
        email
    )
    .fetch_optional(&pool)
    .await
    .context("check if email already verified")?;

    let Some(record) = record else {
        return Err(Error::UnAssociatedEmail(email.clone()));
    };

    if record.email_verified {
        #[cfg(feature = "tracing")]
        tracing::info!("no-op: already verified");

        return Ok(StatusCode::OK);
    }

    let hmac_secret = secrets.get("hmac").context("get HMAC key")?;
    let verification_token = verification_token(email.clone());
    let verification_link = verification_link(&hmac_secret, &host, &verification_token)
        .context("base64 encode email verification link")?;

    let response = send_verification_email(&smtp, &email, &verification_link).await?;
    match response.is_positive() {
        true => {
            #[cfg(feature = "tracing")]
            tracing::info!("{response:?}");

            Ok(StatusCode::OK)
        }
        false => {
            #[cfg(feature = "tracing")]
            tracing::warn!("{response:?}");

            Ok(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidEmailFormat(&'static str),

    #[error("email `{0}` not associated with any user")]
    UnAssociatedEmail(Email),

    #[error("{0}")]
    TokenEncodeError(#[from] contextual::Error<signature::EncodeError>),

    #[error("{0}")]
    SendVerificationEmail(#[from] SendVerificationEmailError),

    #[error("{0}")]
    Io(#[from] contextual::Error<std::io::Error>),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::InvalidEmailFormat(_) => "email.invalid",
            Error::UnAssociatedEmail(_) => "email.unassociated",
            Error::TokenEncodeError(_) => "email.verification.token.encode",
            Error::SendVerificationEmail(_) => "email.verification.send",
            Error::Io(_) => "email.verification.io",
            Error::Sqlx(_) => "email.verification.sqlx",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InvalidEmailFormat(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(self))).into_response()
            }
            Error::UnAssociatedEmail(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::NOT_FOUND, Json(ErrorResponse::from(self))).into_response()
            }
            Error::SendVerificationEmail(err) => err.into_response(),
            Error::TokenEncodeError(_) | Error::Io(_) | Error::Sqlx(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
