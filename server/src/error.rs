use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("PublicError: {0}")]
    Public(#[from] PublicError),

    #[error("InternalError: {0}")]
    Internal(#[from] InternalError),
}

#[derive(thiserror::Error, Debug)]
pub enum PublicError {
    #[error("AuthError: {0}")]
    Auth(#[from] AuthError),

    #[error("CookieError: {0}")]
    Cookie(#[from] CookieError),
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("user '{0}' not found")]
    UserNotFound(String),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("username '{0}' is already taken")]
    UsernameTaken(String),

    #[error("invalid session")]
    InvalidSession,
}

#[derive(thiserror::Error, Debug)]
pub enum CookieError {
    #[error("cookie not found: '{0}'")]
    CookieNotFound(&'static str),
}

#[derive(thiserror::Error, Debug)]
pub enum InternalError {
    #[error("BcryptError: {0:?}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("EnvError: {0:?}")]
    Env(#[from] EnvError),

    #[error("SqlxError: {0:?}")]
    Sqlx(#[from] sqlx::Error),

    #[error("IOError: {0:?}")]
    IO(#[from] std::io::Error),

    #[error("ParseIntError: {0:?}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("ExtensionError: {0:?}")]
    Extension(&'static str),
}

#[derive(thiserror::Error, Debug)]
pub enum EnvError {
    #[error("DotEnvError: {0:?}")]
    DotEnv(#[from] dotenv::Error),

    #[error("VarError: {0:?}")]
    Var(#[from] std::env::VarError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Public(e) => {
                tracing::info!("{}", e);

                let status_code = match &e {
                    PublicError::Auth(e) => {
                        use AuthError::*;
                        match e {
                            UserNotFound(_) => StatusCode::NOT_FOUND,
                            InvalidCredentials | InvalidSession => StatusCode::UNAUTHORIZED,
                            UsernameTaken(_) => StatusCode::CONFLICT,
                        }
                    }
                    PublicError::Cookie(e) => {
                        use CookieError::*;
                        match e {
                            CookieNotFound(_) => StatusCode::BAD_REQUEST,
                        }
                    }
                };

                (status_code, Json(json!({ "message": e.to_string() }))).into_response()
            }
            AppError::Internal(e) => {
                tracing::error!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        AppError::Public(PublicError::Auth(err))
    }
}

impl From<CookieError> for AppError {
    fn from(err: CookieError) -> Self {
        AppError::Public(PublicError::Cookie(err))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::Internal(InternalError::Bcrypt(err))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(InternalError::Sqlx(err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(InternalError::IO(err))
    }
}

impl From<dotenv::Error> for AppError {
    fn from(err: dotenv::Error) -> Self {
        AppError::Internal(InternalError::Env(EnvError::DotEnv(err)))
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        AppError::Internal(InternalError::Env(EnvError::Var(err)))
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::Internal(InternalError::ParseInt(err))
    }
}
