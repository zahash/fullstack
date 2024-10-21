mod access_token;
mod error;
mod health;
mod login;
mod middleware;
mod private;
mod session_id;
mod signup;
mod token;
mod user_id;

use std::{
    collections::VecDeque,
    fmt::Display,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
    usize,
};

use axum::{
    http::{header, Request},
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use forwarded_header_value::{ForwardedHeaderValue, Identifier};
use health::health;
use middleware::{mw_client_ip, mw_handle_leaked_5xx, mw_rate_limiter};
use sqlx::SqlitePool;

use login::login;
use private::private;
use signup::{check_username_availability, signup};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::Span;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub rate_limiter: Arc<RateLimiter>,
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

pub fn ui(path: &str) -> Router {
    Router::<()>::new().nest_service("/", ServeDir::new(path))
}

pub fn server(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/check-username-availability",
            post(check_username_availability),
        )
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/access-token", post(access_token::generate))
        .route("/private", get(private))
        .with_state(state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(mw_handle_leaked_5xx))
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(PropagateRequestIdLayer::x_request_id())
                .layer(from_fn(mw_client_ip))
                .layer(TraceLayer::new_for_http().make_span_with(span))
                .layer(from_fn_with_state(state.clone(), mw_rate_limiter)),
        )
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

fn span<B>(request: &Request<B>) -> Span {
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

fn client_ip<B>(request: &Request<B>) -> Option<IpAddr> {
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
                .get::<SocketAddr>()
                .map(|addr| addr.ip())
        })
}
