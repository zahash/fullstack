use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{MethodRouter, post},
};
use contextual::Context;
use email::Email;
use extra::ErrorResponse;
use serde::Deserialize;
use validation::{validate_password, validate_username};

use crate::{AppState, core::assign_permission_group};

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
    InvalidEmailFormat(&'static str),

    #[error("email `{0}` already linked to another account")]
    EmailExists(Email),

    #[error("{0}")]
    WeakPassword(&'static str),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),

    #[error("{0}")]
    Bcrypt(#[from] contextual::Error<bcrypt::BcryptError>),
}

pub fn method_router() -> MethodRouter<AppState> {
    post(handler)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = PATH,
    operation_id = PATH,
    request_body(
        content = RequestBody,
        content_type = "application/x-www-form-urlencoded",
    ),
    responses(
        (status = 201, description = "User created"),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 409, description = "Username or email already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "auth"
))]
#[cfg_attr(feature = "tracing", tracing::instrument(fields(%username, %email), skip_all, ret))]
pub async fn handler(
    State(AppState {
        pool,

        #[cfg(feature = "smtp")]
        secrets,

        #[cfg(feature = "smtp")]
        smtp,
        ..
    }): State<AppState>,
    #[cfg(feature = "smtp")] axum_extra::extract::Host(host): axum_extra::extract::Host,
    Form(RequestBody {
        username,
        email,
        password,
    }): Form<RequestBody>,
) -> Result<StatusCode, Error> {
    let username = validate_username(username).map_err(Error::InvalidUsername)?;
    let password = validate_password(password).map_err(Error::WeakPassword)?;
    let email = Email::try_from(email).map_err(Error::InvalidEmailFormat)?;

    let mut tx = pool.begin().await.context("begin transaction :: signup")?;

    if super::username::exists(&mut *tx, &username)
        .await
        .context("username exists")?
    {
        return Err(Error::UsernameExists(username));
    }

    if super::email::exists(&mut *tx, &email)
        .await
        .context("email exists")?
    {
        return Err(Error::EmailExists(email));
    }

    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).context("hash password")?;

    let user_id = sqlx::query!(
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
    .fetch_one(&mut *tx)
    .await
    .context("insert user")?
    .user_id;

    assign_permission_group(&mut *tx, user_id, "signup")
        .await
        .context("assign `signup` permission group")?;

    tx.commit().await.context("commit transaction :: signup")?;

    #[cfg(feature = "smtp")]
    {
        use super::email::{
            SendVerificationEmailError, send_verification_email, verification_link,
            verification_token,
        };
        use crate::{secrets::Secrets, smtp::Smtp};
        use lettre::transport::smtp::response::Response as SmtpResponse;

        async fn initiate_email_verification(
            smtp: &Smtp,
            secrets: &Secrets,
            host: &str,
            email: Email,
        ) -> Result<SmtpResponse, InitiateEmailVerificationError> {
            let hmac_secret = secrets.get("hmac").context("get HMAC key")?;
            let verification_token = verification_token(email.clone());
            let verification_link = verification_link(&hmac_secret, &host, &verification_token)
                .context("base64 encode email verification link")?;
            let response = send_verification_email(&smtp, &email, &verification_link).await?;
            Ok(response)
        }

        #[derive(thiserror::Error, Debug)]
        enum InitiateEmailVerificationError {
            #[error("{0}")]
            TokenEncodeError(#[from] contextual::Error<signature::EncodeError>),

            #[error("{0}")]
            SendVerificationEmail(#[from] SendVerificationEmailError),

            #[error("{0}")]
            Io(#[from] contextual::Error<std::io::Error>),
        }

        let _handle = tokio::spawn({
            #[cfg(feature = "tracing")]
            tracing::info!("spawn task to send verification email for {email}");

            let fut = async move {
                let _res = initiate_email_verification(&smtp, &secrets, &host, email).await;

                #[cfg(feature = "tracing")]
                match _res {
                    Ok(response) => match response.is_positive() {
                        true => tracing::info!("{response:?}"),
                        false => tracing::warn!("{response:?}"),
                    },
                    Err(err) => tracing::error!("{err:?}"),
                }
            };

            #[cfg(feature = "tracing")]
            {
                use tracing::Instrument;
                fut.instrument(tracing::Span::current())
            }

            #[cfg(not(feature = "tracing"))]
            fut
        });

        #[cfg(feature = "await-tasks")]
        {
            if let Err(err) = _handle.await {
                #[cfg(feature = "tracing")]
                tracing::error!("failed to await email verification task: {err:?}");
            }
        }
    }

    Ok(StatusCode::CREATED)
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::InvalidUsername(_) => "username.invalid",
            Error::InvalidEmailFormat(_) => "email.invalid",
            Error::WeakPassword(_) => "password.weak",
            Error::UsernameExists(_) => "username.exists",
            Error::EmailExists(_) => "email.exists",
            Error::Sqlx(_) => "sqlx",
            Error::Bcrypt(_) => "bcrypt",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidUsername(_) | Error::InvalidEmailFormat(_) | Error::WeakPassword(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(self))).into_response()
            }
            Error::UsernameExists(_) | Error::EmailExists(_) => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::CONFLICT, Json(ErrorResponse::from(self))).into_response()
            }
            Error::Sqlx(_) | Error::Bcrypt(_) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", self);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
