mod access_token;
mod authorization_header;
mod email;
mod password;
mod permissions;
mod session_id;
mod user_id;
mod username;

pub use access_token::{AccessToken, AccessTokenInfo, AccessTokenValiationError};
pub use authorization_header::{AuthorizationHeader, AuthorizationHeaderError};
pub use email::Email;
pub use password::Password;
pub use permissions::{InsufficientPermissionsError, Permissions, Principal};
pub use session_id::{SessionExt, SessionId, SessionInfo, SessionValidationError};
pub use user_id::UserId;
pub use username::Username;

pub struct Valid<T>(T);

#[derive(thiserror::Error, Debug)]
#[error("cannot base64 decode :: {0}")]
pub struct Base64DecodeError(&'static str);
