use auth::assign_permission_group;
use axum::{
    Form, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use contextual::Context;
use email::Email;
use extra::ErrorResponse;
use serde::Deserialize;

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
    Sqlx(#[from] contextual::Error<sqlx::Error>),

    #[error("{0}")]
    Bcrypt(#[from] contextual::Error<bcrypt::BcryptError>),
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

    let mut tx = pool.begin().await.context("begin transaction")?;

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

    tx.commit().await.context("commit transaction")?;

    #[cfg(feature = "smtp")]
    tokio::spawn({
        #[cfg(feature = "tracing")]
        tracing::info!("spawn task to initiate email verification for {email}");

        let fut = async move {
            let _res = crate::smtp::initiate_email_verification(&pool, &smtp, &email)
                .await
                .context("signup");

            #[cfg(feature = "tracing")]
            match _res {
                Ok(response) => {
                    tracing::info!("initiate_email_verification response :: {response:?}")
                }
                Err(err) => tracing::error!("initiate_email_verification error :: {err:?}"),
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

    Ok(StatusCode::CREATED)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::InvalidUsername(_) | Error::InvalidEmail(_) | Error::WeakPassword(_) => {
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
