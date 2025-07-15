mod access_token;
pub use access_token::{
    AccessToken, AccessTokenAuthorizationExtractionError, AccessTokenInfo,
    AccessTokenValidationError,
};

mod basic;
pub use basic::{Basic, BasicAuthorizationExtractionError};

mod credentials;
pub use credentials::Credentials;

mod permission;
pub use permission::{InsufficientPermissionsError, Permission, Permissions};

mod principal;
pub use principal::{Principal, PrincipalError};

mod session;
pub use session::{
    SessionCookieExtractionError, SessionId, SessionInfo, SessionValidationError,
    expired_session_cookie,
};

mod user;
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
