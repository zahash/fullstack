mod access_token;
mod basic;
mod credentials;
mod permission;
mod principal;
mod session;
mod user;

pub use access_token::{
    AccessToken, AccessTokenAuthorizationExtractionError, AccessTokenInfo,
    AccessTokenValidationError,
};
pub use basic::{Basic, BasicAuthorizationExtractionError};
pub use credentials::Credentials;
pub use permission::{Authorizable, InsufficientPermissionsError, Permission};
pub use principal::{Principal, PrincipalError};
pub use session::{
    SessionCookieExtractionError, SessionId, SessionInfo, SessionValidationError,
    expired_session_cookie,
};
pub use user::UserInfo;

pub struct Verified<T>(T);

impl<T> Verified<T> {
    #[inline]
    pub fn inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Verified<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Verified<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

pub async fn assign_permission_group<'a, E: sqlx::Executor<'a, Database = sqlx::Sqlite>>(
    ex: E,
    user_id: i64,
    group: &str,
) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO user_permissions (user_id, permission_id)

        SELECT ? AS user_id, pga.permission_id FROM permission_groups pg
        INNER JOIN permission_group_association pga ON pg.id = pga.permission_group_id

        WHERE pg.[group] = ?

        ON CONFLICT(user_id, permission_id) DO NOTHING;
        "#,
        user_id,
        group
    )
    .execute(ex)
    .await
}
