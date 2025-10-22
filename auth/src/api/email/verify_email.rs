use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use axum_macros::debug_handler;
use contextual::Context;
use email::Email;
use extra::ErrorResponse;
use http::StatusCode;
use serde::Deserialize;

use crate::AppState;

pub const PATH: &str = "/verify-email";

#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
#[cfg_attr(feature = "openapi", into_params(parameter_in = Query))]
#[derive(Deserialize)]
pub struct QueryParams {
    pub token: String,
}

pub fn method_router() -> MethodRouter<AppState> {
    get(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    operation_id = PATH,
    params(QueryParams),
    responses(
        (status = 200, description = "Email verified successfully"),
        (status = 400, description = "Invalid or malformed token", body = ErrorResponse),
        (status = 404, description = "Token not found", body = ErrorResponse),
        (status = 410, description = "Token expired", body = ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "email"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(email = tracing::field::Empty), skip_all, ret))]
#[debug_handler]
pub async fn handler(
    State(AppState { pool, secrets, .. }): State<AppState>,
    Query(QueryParams {
        token: token_base64_encoded,
    }): Query<QueryParams>,
) -> Result<StatusCode, Error> {
    let hmac_secret = secrets.get("hmac").context("get HMAC key")?;
    let signed_token = signature::Signed::<Email>::decode(&token_base64_encoded, &hmac_secret)?;
    let email = signed_token.token()?;

    #[cfg(feature = "tracing")]
    tracing::Span::current().record("email", tracing::field::display(&email));

    sqlx::query!("UPDATE users SET email_verified = 1 WHERE email = ?", email)
        .execute(&pool)
        .await
        .context("user email_verified")?;

    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    TokenDecode(#[from] signature::DecodeError<email::ParseError>),

    #[error("{0}")]
    TemporalTokenValidity(#[from] signature::TemporalValidityError),

    #[error("{0}")]
    Io(#[from] contextual::Error<std::io::Error>),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::TokenDecode(_) => "token.decode",
            Error::TemporalTokenValidity(_) => "token.validity",
            Error::Io(_) => "io",
            Error::Sqlx(_) => "sqlx",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::TokenDecode(decode_error) => match decode_error {
                signature::DecodeError::InvalidFormat => {
                    #[cfg(feature = "tracing")]
                    tracing::info!("{:?}", decode_error);

                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse::new(
                            decode_error.to_string(),
                            "email.invalid",
                        )),
                    )
                        .into_response()
                }
                signature::DecodeError::MacMismatch(_)
                | signature::DecodeError::NonUTF8(_)
                | signature::DecodeError::Serde(_)
                | signature::DecodeError::Base64(_)
                | signature::DecodeError::TokenFromBytes(_) => {
                    #[cfg(feature = "tracing")]
                    tracing::info!("{:?}", decode_error);

                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse::new(
                            "Invalid Verification Token",
                            "token.invalid",
                        )),
                    )
                        .into_response()
                }
                signature::DecodeError::InvalidKeyLength => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("{:?}", decode_error);

                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            },
            Error::TemporalTokenValidity(err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", err);

                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new(
                        err.to_string(),
                        "token.temporal.invalid",
                    )),
                )
                    .into_response()
            }
            Error::Io(_) | Error::Sqlx(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
