use auth::{Credentials, SessionId, expired_session_cookie};
use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use contextual::Context;
use http::{HeaderMap, StatusCode};

use crate::AppState;

pub const PATH: &str = "/logout";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = PATH,
    operation_id = PATH,
    responses((status = 200, description = "Session invalidated and Cookie removed")),
    tag = "auth"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(user_id = tracing::field::Empty), skip_all))]
#[debug_handler]
pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar), Error> {
    if let Ok(Some(session_id)) = SessionId::try_from_headers(&headers) {
        let session_id_hash = session_id.hash_sha256();

        let _record = sqlx::query!(
            r#"
            DELETE FROM sessions WHERE session_id_hash = ?
            RETURNING user_id
            "#,
            session_id_hash
        )
        .fetch_one(&pool)
        .await
        .context("delete session")?;

        #[cfg(feature = "tracing")]
        tracing::Span::current().record("user_id", tracing::field::display(_record.user_id));
    }

    let jar = jar.add(expired_session_cookie());
    Ok((StatusCode::OK, jar))
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
