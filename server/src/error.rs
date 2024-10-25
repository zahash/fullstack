use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub const HELP: &'static str = "Please check the response headers for `x-request-id`, include the datetime and raise a support ticket.";
pub const SECURITY: &'static str = "Security incident detected! This will be reported immediately!";

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
