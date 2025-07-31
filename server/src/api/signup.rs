use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use contextual::Context;
use email::Email;
use extra::json_error_response;
use serde::Deserialize;

use tag::Tag;
use validation::{validate_password, validate_username};

use crate::AppState;

pub const PATH: &str = "/signup";

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = signup::RequestBody))]
#[derive(Deserialize)]
pub struct RequestBody {
    #[cfg_attr(feature = "openapi", schema(examples("joe")))]
    pub username: String,

    #[cfg_attr(feature = "openapi", schema(examples("joe@smith.com")))]
    pub email: String,

    #[cfg_attr(feature = "openapi", schema(examples("h?P7o]37")))]
    pub password: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidUsername(&'static str),

    #[error("username `{0}` is not available")]
    UsernameExists(String),

    #[error("{0}")]
    InvalidEmail(&'static str),

    #[error("email `{0}` already linked to another account")]
    EmailExists(Email),

    #[error("{0}")]
    WeakPassword(&'static str),

    #[error("{0}")]
    DataAccess(#[from] contextual::Error<data_access::Error>),

    #[error("{0}")]
    Bcrypt(#[from] contextual::Error<bcrypt::BcryptError>),
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = PATH,
    request_body(
        content = RequestBody,
        content_type = "application/x-www-form-urlencoded",
    ),
    responses(
        (status = 201, description = "User created"),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Username or email already exists"),
        (status = 500, description = "Internal server error"),
    )
))]
#[tracing::instrument(fields(%username, %email), skip_all, ret)]
pub async fn handler(
    State(AppState {
        data_access,

        #[cfg(feature = "smtp")]
        smtp,
    }): State<AppState>,
    Form(RequestBody {
        username,
        email,
        password,
    }): Form<RequestBody>,
) -> Result<StatusCode, Error> {
    let username = validate_username(username).map_err(Error::InvalidUsername)?;
    let password = validate_password(password).map_err(Error::WeakPassword)?;
    let email = Email::try_from(email).map_err(Error::InvalidEmail)?;

    if super::username::exists(&data_access, &username)
        .await
        .context("username exists")?
    {
        return Err(Error::UsernameExists(username));
    }

    if super::email::exists(&data_access, &email)
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
                    Tag {
                        table: "users",
                        primary_key: None,
                    },
                    Tag {
                        table: "users",
                        primary_key: Some(value.user_id),
                    },
                ]
            },
        )
        .await
        .context("insert user")?;

    #[cfg(feature = "smtp")]
    tokio::spawn({
        use tracing::Instrument;
        tracing::info!("spawn task to initiate email verification for {email}");

        async move {
            match crate::smtp::initiate_email_verification(&data_access, &smtp, &email)
                .await
                .context("signup")
            {
                Ok(response) => {
                    tracing::info!("initiate_email_verification response :: {response:?}")
                }
                Err(err) => tracing::error!("initiate_email_verification error :: {err:?}"),
            }
        }
        .instrument(tracing::Span::current())
    });

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
            Error::DataAccess(_) | Error::Bcrypt(_) => {
                tracing::error!("{:?}", self);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
