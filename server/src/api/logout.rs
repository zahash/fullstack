use auth::{Credentials, SessionId, expired_session_cookie};
use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use contextual::Context;
use http::{HeaderMap, StatusCode};
use tag::Tag;

use crate::AppState;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    DataAccess(#[from] contextual::Error<data_access::Error>),
}

#[debug_handler]
pub async fn logout(
    State(AppState { data_access, .. }): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar), Error> {
    if let Ok(Some(session_id)) = SessionId::try_from_headers(&headers) {
        let session_id_hash = session_id.hash_sha256();

        data_access
            .write(
                |pool| {
                    sqlx::query!(
                        r#"
                        DELETE FROM sessions WHERE session_id_hash = ?
                        RETURNING id as "id!"
                        "#,
                        session_id_hash
                    )
                    .fetch_one(pool)
                },
                |value| {
                    vec![
                        Tag {
                            table: "sessions",
                            primary_key: None,
                        },
                        Tag {
                            table: "sessions",
                            primary_key: Some(value.id),
                        },
                    ]
                },
            )
            .await
            .context("delete session")?;
    }

    let jar = jar.add(expired_session_cookie());
    Ok((StatusCode::OK, jar))
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::DataAccess(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
