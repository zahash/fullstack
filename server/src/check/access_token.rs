use axum::http::StatusCode;
use axum_macros::debug_handler;

use crate::access_token::AccessToken;

#[debug_handler]
#[tracing::instrument(skip_all, ret)]
/// - If the `AccessToken` extractor successfully extracts and validates the token,
///   the function proceeds and returns `StatusCode::OK`.
/// - If extraction or validation fails, the request is rejected with an appropriate
///   error status before reaching this point.
pub async fn access_token(_: AccessToken) -> StatusCode {
    StatusCode::OK
}
