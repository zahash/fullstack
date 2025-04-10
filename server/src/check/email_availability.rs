use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    AppState,
    check::email_exists,
    error::{Context, HELP, InternalError},
    misc::now_iso8601,
    types::Email,
};

#[derive(Deserialize)]
pub struct Params {
    pub email: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidParams(&'static str),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[tracing::instrument(fields(?email), skip_all, ret)]
pub async fn email_availability(
    State(AppState { pool, .. }): State<AppState>,
    Query(Params { email }): Query<Params>,
) -> Result<StatusCode, Error> {
    let email = Email::try_from(email).map_err(Error::InvalidParams)?;

    match email_exists(&pool, &email)
        .await
        .context("check email availability")?
    {
        true => Ok(StatusCode::CONFLICT),
        false => Ok(StatusCode::OK),
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InvalidParams(err) => {
                tracing::info!("{:?}", err);
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": err,
                        "help": HELP,
                        "datetime": now_iso8601()
                    })),
                )
                    .into_response()
            }
            Error::Internal(err) => err.into_response(),
        }
    }
}
