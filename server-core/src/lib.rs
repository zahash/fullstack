mod access_token;
mod authorization_header;
mod email;
mod error;
mod middleware;
mod password;
mod permissions;
mod session_id;
mod token;
mod user_id;
mod username;

pub use access_token::{AccessToken, AccessTokenInfo, AccessTokenValiationError};
pub use authorization_header::{AuthorizationHeader, AuthorizationHeaderError};
pub use email::Email;
pub use error::{Context, InternalError, error};
pub use middleware::{mw_client_ip, mw_handle_leaked_5xx, mw_rate_limiter};
pub use password::Password;
pub use permissions::{InsufficientPermissionsError, Permissions, Principal};
pub use session_id::{SessionExt, SessionId, SessionInfo, SessionValidationError};
pub use token::Token;
pub use user_id::UserId;
pub use username::Username;

use std::{
    collections::VecDeque,
    fmt::Display,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::ConnectInfo,
    http::{Request, header},
};
use dashmap::DashMap;
use forwarded_header_value::{ForwardedHeaderValue, Identifier};
use tracing::Span;
// use lettre::SmtpTransport;
use sqlx::SqlitePool;

// TODO
// email verification during signup
// shared validation code between frontend and backend as wasm (is_strong_password)

pub struct AppState {
    pub pool: SqlitePool,
    pub rate_limiter: Arc<RateLimiter>,
    // pub mailer: Arc<()>,
}

pub struct RateLimiter {
    requests: DashMap<IpAddr, VecDeque<Instant>>,
    limit: usize,
    interval: Duration,
}

impl RateLimiter {
    pub fn new(limit: usize, interval: Duration) -> Self {
        Self {
            requests: DashMap::default(),
            limit,
            interval,
        }
    }

    pub fn nolimit() -> Self {
        Self {
            requests: DashMap::default(),
            limit: usize::MAX,
            interval: Duration::from_secs(0),
        }
    }

    pub fn is_too_many(&self, ip_addr: IpAddr) -> bool {
        let now = Instant::now();
        let mut request_timeline = self.requests.entry(ip_addr).or_insert_with(VecDeque::new);

        // clean up old entries
        while let Some(&time) = request_timeline.front() {
            if now.duration_since(time) < self.interval {
                break;
            }
            request_timeline.pop_front();
        }

        if request_timeline.len() > self.limit {
            return true;
        }

        request_timeline.push_back(now);
        false
    }
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

#[derive(thiserror::Error, Debug)]
#[error("cannot base64 decode :: {0}")]
pub struct Base64DecodeError(&'static str);

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            rate_limiter: Arc::clone(&self.rate_limiter),
            // mailer: Arc::clone(&self.mailer),
        }
    }
}
