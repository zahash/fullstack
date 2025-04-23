use std::fmt::Display;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use time::{OffsetDateTime, format_description::well_known::Iso8601};

const HELP: &'static str = "Please check the response headers for `x-request-id`, include the datetime and raise a support ticket.";
// pub const SECURITY: &'static str = "Security incident detected! This will be reported immediately!";

#[derive(thiserror::Error, Debug)]
#[error("{0:?}")]
pub struct InternalError(#[from] pub anyhow::Error);

impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub trait Context<T> {
    fn context<C>(self, context: C) -> Result<T, InternalError>
    where
        C: Display + Send + Sync + 'static;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: Into<anyhow::Error> + std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T, InternalError>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| InternalError(anyhow::Error::from(e).context(context)))
    }
}

fn now_iso8601() -> Option<String> {
    OffsetDateTime::now_utc()
        .format(&Iso8601::DATE_TIME_OFFSET)
        .inspect_err(|e| {
            tracing::warn!(
                "unable to format OffsetDateTime::now_utc() as Iso8601 :: {:?}",
                e
            )
        })
        .ok()
}

pub fn error(msg: &str) -> Json<serde_json::Value> {
    Json(json!({
        "error": msg,
        "help": HELP,
        "datetime": now_iso8601()
    }))
}

// pub fn security_error(msg: &str) -> Json<serde_json::Value> {
//     Json(json!({
//         "error": msg,
//         "security": SECURITY,
//         "datetime": now_iso8601()
//     }))
// }
