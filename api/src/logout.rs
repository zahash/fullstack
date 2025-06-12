use axum::extract::State;
use axum_extra::extract::CookieJar;

use server_core::{AppState, Context, InternalError, SessionExt, SessionId};

pub async fn logout(
    State(AppState { data_access, .. }): State<AppState>,
    mut jar: CookieJar,
) -> Result<CookieJar, InternalError> {
    if let Ok(Some(session_id)) = SessionId::try_from_cookie_jar(&jar) {
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

        jar = jar.remove_session_cookie();
    }

    Ok(jar)
}
