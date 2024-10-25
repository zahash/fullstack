use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Form, Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    check::{email_exists, username_exists},
    error::{Context, InternalError, HELP},
    misc::now_iso8601,
    types::{Email, Password, Username},
    AppState,
};

#[derive(Deserialize)]
pub struct SignUp {
    pub username: Username,
    pub email: Email,
    pub password: Password,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0} is not available")]
    UsernameExists(Username),

    #[error("{0} already linked to another account")]
    EmailExists(Email),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[tracing::instrument(fields(?username), skip_all, ret)]
pub async fn signup<T>(
    State(AppState { pool, .. }): State<AppState<T>>,
    Form(SignUp {
        username,
        email,
        password,
    }): Form<SignUp>,
) -> Result<StatusCode, Error> {
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).context("hash password")?;

    if username_exists(&pool, &username)
        .await
        .context("username exists")?
    {
        return Err(Error::UsernameExists(username));
    }

    if email_exists(&pool, &email).await.context("email exists")? {
        return Err(Error::EmailExists(email));
    }

    sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)",
        username,
        email,
        password_hash,
    )
    .execute(&pool)
    .await
    .context("insert user")?;

    Ok(StatusCode::CREATED)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::UsernameExists(_) | Error::EmailExists(_) => {
                tracing::info!("{:?}", self);
                (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": self.to_string(),
                        "help": HELP,
                        "datetime": now_iso8601(),
                    })),
                )
                    .into_response()
            }
            Error::Internal(err) => {
                tracing::warn!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
