mod access_token;
pub use access_token::{AccessToken, AccessTokenInfo, AccessTokenValidationError};

mod authorization_header;
pub use authorization_header::{AuthorizationHeader, AuthorizationHeaderError};

mod permission;
pub use permission::{InsufficientPermissionsError, Permission, Permissions};

mod session;
pub use session::{SessionExt, SessionId, SessionInfo, SessionValidationError};

mod user;
pub use user::UserInfo;

use std::ops::Deref;

pub struct Verified<T>(T);

impl<T> Verified<T> {
    #[inline]
    pub fn inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Verified<T> {
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

#[derive(thiserror::Error, Debug)]
#[error("cannot base64 decode :: {0}")]
pub struct Base64DecodeError(&'static str);
