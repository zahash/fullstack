use axum::{
    Form,
    extract::{State, rejection::FormRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use server_core::{AppState, Context, Email, InternalError, Password, Username, error};

use crate::{email::email_exists, username::username_exists};

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

    #[error("{0}")]
    FormRejection(FormRejection),

    #[error("{0:?}")]
    Internal(#[from] InternalError),
}

#[tracing::instrument(fields(username = tracing::field::Empty), skip_all, ret)]
pub async fn signup(
    State(AppState { pool, .. }): State<AppState>,
    payload: Result<Form<SignUp>, FormRejection>,
) -> Result<StatusCode, Error> {
    let Form(SignUp {
        username,
        email,
        password,
    }) = payload.map_err(Error::from)?;

    tracing::Span::current().record("username", &tracing::field::display(&username));

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

impl From<FormRejection> for Error {
    fn from(err: FormRejection) -> Self {
        Error::FormRejection(err)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::FormRejection(err) => {
                tracing::info!("{:?}", err);
                (err.status(), error(&err.body_text())).into_response()
            }
            Error::UsernameExists(_) | Error::EmailExists(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::CONFLICT, error(&self.to_string())).into_response()
            }
            Error::Internal(err) => err.into_response(),
        }
    }
}
