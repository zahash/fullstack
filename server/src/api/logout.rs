use auth::{Credentials, SessionId};
use axum::{extract::State, response::IntoResponse};
use axum_macros::debug_handler;
use boxer::{Boxer, Context};
use http::{HeaderMap, StatusCode};

use crate::AppState;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0:?}")]
    Internal(#[from] Boxer),
}

#[debug_handler]
pub async fn logout(
    State(AppState { data_access, .. }): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, Error> {
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
                        Box::new("sessions"),
                        Box::new(format!("sessions:{}", value.id)),
                    ]
                },
            )
            .await
            .context("delete session")?;
    }

    Ok(StatusCode::OK)
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Internal(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
