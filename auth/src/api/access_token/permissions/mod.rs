pub mod assign;
pub mod revoke;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
    routing::{MethodRouter, get},
};
use contextual::Context;
use extra::ErrorResponse;
use http::StatusCode;
use serde::Deserialize;

use crate::{
    AppState,
    core::{
        AccessTokenInfo, AccessTokenValidationError, Authorizable, InsufficientPermissionsError,
        Permission, Principal,
    },
};

pub const PATH: &str = "/access-token/permissions";

#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
#[cfg_attr(feature = "openapi", into_params(parameter_in = Query))]
#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[cfg_attr(feature = "openapi", param(example = "my-token"))]
    pub token_name: String,
}

pub fn method_router() -> MethodRouter<AppState> {
    get(handler)
}

pub async fn handler(
    State(AppState { pool, .. }): State<AppState>,
    Query(QueryParams { token_name }): Query<QueryParams>,
    principal: Principal,
) -> Result<Json<Vec<Permission>>, Error> {
    principal
        .require_permission::<Error>(&pool, "get:/access-token/permissions")
        .await?;

    if let Principal::AccessToken(info) = &principal
        && info.name == token_name
    {
        let permissions = info
            .permissions(&pool)
            .await
            .context("get access token permissions")?;
        return Ok(Json(permissions));
    }

    let user_id = principal.user_id();

    let access_token_info = sqlx::query_as!(
        AccessTokenInfo,
        r#"
        SELECT id as "id!", name, user_id, created_at, expires_at
        FROM access_tokens
        WHERE user_id = ? AND name = ?
        "#,
        user_id,
        token_name
    )
    .fetch_optional(&pool)
    .await
    .context("fetch access token info")?
    .ok_or(Error::NotFound)?;

    let verified_info = access_token_info.verify()?;

    let permissions = verified_info
        .permissions(&pool)
        .await
        .context("get access token permissions")?;
    Ok(Json(permissions))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InsufficientPermissions(#[from] InsufficientPermissionsError),

    #[error("access token not found")]
    NotFound,

    #[error("{0}")]
    AccessTokenValidation(#[from] AccessTokenValidationError),

    #[error("{0}")]
    Sqlx(#[from] contextual::Error<sqlx::Error>),
}

impl extra::ErrorKind for Error {
    fn kind(&self) -> &'static str {
        match self {
            Error::InsufficientPermissions(e) => e.kind(),
            Error::NotFound => "access-token.not-found",
            Error::AccessTokenValidation(e) => e.kind(),
            Error::Sqlx(_) => "sqlx",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InsufficientPermissions(err) => err.into_response(),
            Error::NotFound => {
                #[cfg(feature = "tracing")]
                tracing::info!("{:?}", self);

                (StatusCode::NOT_FOUND, Json(ErrorResponse::from(self))).into_response()
            }
            Error::AccessTokenValidation(_err) => todo!(),
            Error::Sqlx(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("{:?}", _err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
