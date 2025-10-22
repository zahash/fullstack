use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use contextual::Context;
use email::Email;
use extra::ErrorResponse;
use serde::Deserialize;

use crate::AppState;

pub const PATH: &str = "/check/email-availability";

#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
#[cfg_attr(feature = "openapi", into_params(parameter_in = Query))]
#[derive(Deserialize)]
pub struct QueryParams {
    #[cfg_attr(feature = "openapi", param(example = "joe@smith.com"))]
    pub email: String,
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
        (status = 200, description = "Email is available"),
        (status = 409, description = "Email already exists"),
        (status = 400, description = "Invalid email address", body = ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "check"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%email), skip_all, ret))]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    Query(QueryParams { email }): Query<QueryParams>,
) -> Result<StatusCode, Error> {
    let email = Email::try_from(email).map_err(Error::InvalidParams)?;

    match super::exists(&pool, &email)
        .await
        .context("check email availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::InvalidParams(_) => "email.invalid",
            Error::Sqlx(_) => "sqlx",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InvalidParams(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(self))).into_response()
            }
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
