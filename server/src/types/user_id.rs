use sqlx::{Sqlite, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct UserId(i64);

// #[derive(thiserror::Error, Debug)]
// pub enum Error {
//     #[error("{0}")]
//     Auth(#[from] AuthError),
// }

// impl FromRequestParts<AppState> for UserId {
//     type Rejection = Error;

//     async fn from_request_parts(
//         parts: &mut Parts,
//         state: &AppState,
//     ) -> Result<Self, Self::Rejection> {
//         match Auth::from_request_parts(parts, state).await? {
//             Auth::Session { user_id, .. } | Auth::AccessToken { user_id, .. } => Ok(user_id),
//         }
//     }
// }

impl Type<Sqlite> for UserId {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, Sqlite> for UserId {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as sqlx::Encode<Sqlite>>::encode_by_ref(&self.0, buf)
    }
}

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        UserId(value)
    }
}

// impl IntoResponse for Error {
//     fn into_response(self) -> Response {
//         match self {
//             Error::Auth(err) => err.into_response(),
//         }
//     }
// }
