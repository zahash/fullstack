use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
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
    pub username: String,
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    params(QueryParams),
    responses(
        (status = 200, description = "Username is available"),
        (status = 409, description = "Username is already taken"),
        (status = 400, description = "Invalid username format", body = ErrorResponse),
    ),
    tag = "check"
))]
#[tracing::instrument(fields(%username), skip_all, ret)]
pub async fn handler(
    State(AppState { data_access, .. }): State<AppState>,
    Query(QueryParams { username }): Query<QueryParams>,
) -> Result<StatusCode, Error> {
    let username = validate_username(username).map_err(Error::InvalidParams)?;

    match super::exists(&data_access, &username)
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
    DataAccess(#[from] contextual::Error<data_access::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InvalidParams(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(self))).into_response()
            }
            Error::DataAccess(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
