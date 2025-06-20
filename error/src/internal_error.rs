#[derive(thiserror::Error, Debug)]
#[error("{0:?}")]
pub struct InternalError(#[from] pub anyhow::Error);

#[cfg(feature = "internal-error-axum")]
impl axum::response::IntoResponse for InternalError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self.0);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
