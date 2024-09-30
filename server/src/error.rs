use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Public(#[from] PublicError),

    #[error("{0:?}")]
    Internal(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum PublicError {
    #[error("{0}")]
    Auth(#[from] AuthError),

    #[error("{0}")]
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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Public(e) => {
                tracing::info!("{:?}", e);

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

                (
                    status_code,
                    Json(json!(
                    {
                        "message": e.to_string(),
                    })),
                )
                    .into_response()
            }
            AppError::Internal(e) => {
                tracing::error!("{:?}", e);
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
