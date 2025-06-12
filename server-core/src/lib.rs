mod access_token;
mod auth;
mod authorization_header;
mod data_access;
mod email;
mod error;
mod middleware;
mod rate_limiter;
mod session;
mod token;
mod user;

pub use access_token::{AccessToken, AccessTokenInfo, AccessTokenValiationError};
pub use auth::{InsufficientPermissionsError, Permission, Permissions, Principal};
pub use authorization_header::{AuthorizationHeader, AuthorizationHeaderError};
pub use data_access::DataAccess;
pub use email::Email;
pub use error::{Context, InternalError, error};
pub use middleware::{mw_client_ip, mw_handle_leaked_5xx, mw_rate_limiter};
pub use rate_limiter::RateLimiter;
pub use session::{SessionExt, SessionId, SessionInfo, SessionValidationError};
pub use token::Token;
pub use user::UserInfo;

use std::{
    fmt::Display,
    net::{IpAddr, SocketAddr},
    ops::Deref,
    str::FromStr,
    sync::Arc,
};

use axum::{
    extract::ConnectInfo,
    http::{Request, header},
};
use forwarded_header_value::{ForwardedHeaderValue, Identifier};
use tracing::Span;
// use lettre::SmtpTransport;

// TODO
// email verification during signup
// shared validation code between frontend and backend as wasm (is_strong_password)

pub struct AppState {
    pub data_access: DataAccess,
    pub rate_limiter: Arc<RateLimiter>,
    // pub mailer: Arc<()>,
}

struct OptionDisplay<T>(Option<T>, &'static str);

impl<T: Display> Display for OptionDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(val) => write!(f, "{}", val),
            None => write!(f, "{}", self.1),
        }
    }
}

pub fn span<B>(request: &Request<B>) -> Span {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("<unknown-request-id>");

    let client_ip = request
        .extensions()
        .get::<Option<IpAddr>>()
        .copied()
        .flatten();

    tracing::info_span!(
        "request",
        "{} {} {} {}",
        OptionDisplay(client_ip, "<unknown-client-ip>"),
        request_id,
        request.method(),
        request.uri(),
    )
}

pub fn client_ip<B>(request: &Request<B>) -> Option<IpAddr> {
    request
        .headers()
        .get(header::FORWARDED)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| ForwardedHeaderValue::from_str(val).ok())
        .map(|forwarded| forwarded.into_remotest())
        .and_then(|stanza| stanza.forwarded_for)
        .and_then(|identifier| match identifier {
            Identifier::SocketAddr(socket_addr) => Some(socket_addr.ip()),
            Identifier::IpAddr(ip_addr) => Some(ip_addr),
            _ => None,
        })
        .or_else(|| {
            request
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|connect_info| connect_info.0.ip())
        })
}

pub struct Valid<T>(T);

impl<T> Valid<T> {
    #[inline]
    pub fn inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Valid<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Valid<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

#[derive(thiserror::Error, Debug)]
#[error("cannot base64 decode :: {0}")]
pub struct Base64DecodeError(&'static str);

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            data_access: self.data_access.clone(),
            rate_limiter: Arc::clone(&self.rate_limiter),
            // mailer: Arc::clone(&self.mailer),
        }
    }
}
