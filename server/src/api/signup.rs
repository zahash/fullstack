use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use boxer::{Boxer, Context};
use email::Email;
use extra::json_error_response;
use serde::Deserialize;

use validation::{validate_password, validate_username};

use super::{email::email_exists, username::username_exists};
use crate::AppState;

#[derive(Deserialize)]
pub struct SignUp {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidUsername(&'static str),

    #[error("{0} is not available")]
    UsernameExists(String),

    #[error("{0}")]
    InvalidEmail(&'static str),

    #[error("{0} already linked to another account")]
    EmailExists(Email),

    #[error("{0}")]
    WeakPassword(&'static str),

    #[error("{0:?}")]
    Internal(#[from] Boxer),
}

#[tracing::instrument(fields(username = tracing::field::Empty), skip_all, ret)]
pub async fn signup(
    State(AppState { data_access, .. }): State<AppState>,
    Form(SignUp {
        username,
        email,
        password,
    }): Form<SignUp>,
) -> Result<StatusCode, Error> {
    let username = validate_username(username).map_err(Error::InvalidUsername)?;
    let password = validate_password(password).map_err(Error::WeakPassword)?;
    let email = Email::try_from(email).map_err(Error::InvalidEmail)?;

    tracing::Span::current().record("username", tracing::field::display(&username));

    if username_exists(&data_access, &username)
        .await
        .context("username exists")?
    {
        return Err(Error::UsernameExists(username));
    }

    if email_exists(&data_access, &email)
        .await
        .context("email exists")?
    {
        return Err(Error::EmailExists(email));
    }

    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).context("hash password")?;

    data_access
        .write(
            |pool| {
                sqlx::query!(
                    r#"
                    INSERT INTO users
                    (username, email, password_hash)
                    VALUES (?, ?, ?)
                    RETURNING id as "user_id!"
                    "#,
                    username,
                    email,
                    password_hash,
                )
                .fetch_one(pool)
            },
            |value| {
                vec![
                    Box::new("users"),
                    Box::new(format!("users:{}", value.user_id)),
                ]
            },
        )
        .await
        .context("insert user")?;

    Ok(StatusCode::CREATED)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidUsername(_) | Error::InvalidEmail(_) | Error::WeakPassword(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::BAD_REQUEST, Json(json_error_response(self))).into_response()
            }
            Error::UsernameExists(_) | Error::EmailExists(_) => {
                tracing::info!("{:?}", self);
                (StatusCode::CONFLICT, Json(json_error_response(self))).into_response()
            }
            Error::Internal(err) => {
                tracing::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
