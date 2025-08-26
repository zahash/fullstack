use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use contextual::Context;
use extra::ErrorResponse;
use serde::Deserialize;
use validation::validate_username;

use crate::AppState;

pub const PATH: &str = "/check/username-availability";

#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
#[cfg_attr(feature = "openapi", into_params(parameter_in = Query))]
#[derive(Deserialize)]
pub struct QueryParams {
    #[cfg_attr(feature = "openapi", param(example = "joe"))]
    pub username: String,
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
        (status = 200, description = "Username is available"),
        (status = 409, description = "Username is already taken"),
        (status = 400, description = "Invalid username format", body = ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "check"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%username), skip_all, ret))]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    Query(QueryParams { username }): Query<QueryParams>,
) -> Result<StatusCode, Error> {
    let username = validate_username(username).map_err(Error::InvalidParams)?;

    match super::exists(&pool, &username)
        .await
        .context("check username availability")?
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
            Error::InvalidParams(_) => "username.invalid",
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
