mod access_token;
mod error;
mod health;
mod login;
mod private;
mod request_id;
mod session_id;
mod signup;
mod token;
mod user_id;

use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
    usize,
};

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use forwarded_header_value::{ForwardedHeaderValue, Identifier};
use health::health;
use request_id::RequestId;
use sqlx::SqlitePool;

use login::login;
use private::private;
use signup::{check_username_availability, signup};
use tower_http::{services::ServeDir, trace::TraceLayer};

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

    pub fn check(&self, ip_addr: IpAddr) -> bool {
        let now = Instant::now();
        let mut entry = self.requests.entry(ip_addr).or_insert_with(VecDeque::new);

        // clean up old entries
        while let Some(&time) = entry.front() {
            if now.duration_since(time) < self.interval {
                break;
            }
            entry.pop_front();
        }

        if entry.len() > self.limit {
            return false;
        }

        entry.push_back(now);
        true
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
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            |State(AppState { rate_limiter, .. }): State<AppState>, request: Request<Body>, next: Next| async move {
                    let client_ip = request
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
                        .unwrap_or_else(|| {
                            request
                                .extensions()
                                .get::<SocketAddr>()
                                .map(|addr| addr.ip())
                                .unwrap_or_else(|| {
                                    tracing::warn!("unable to get SocketAddr from request");
                                    Ipv4Addr::new(0, 0, 0, 0).into()
                                })
                        });

                    match rate_limiter.check(client_ip) {
                        true => next.run(request).await,
                        false => StatusCode::TOO_MANY_REQUESTS.into_response(),
                    }
                }
            ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| match request
                .extensions()
                .get::<RequestId>() {
                    Some(request_id) => tracing::info_span!(
                        "request",
                        "{} {} {}",
                        request_id,
                        request.method(),
                        request.uri(),
                    ),
                    None => tracing::info_span!(
                        "request",
                        "{} {}",
                        request.method(),
                        request.uri(),
                    ),
                }
            ),
        )
        .layer(axum::middleware::from_fn(
            |mut request: Request<Body>, next: Next| async {
                let request_id = RequestId::new();
                request.extensions_mut().insert(request_id);
                next.run(request).await
            },
        ))
        .layer(axum::middleware::from_fn(
            |request: Request<Body>, next: Next| async {
                let response = next.run(request).await;
                let status = response.status();

                match status.is_server_error() {
                    false => response,
                    true => {
                        // Log and capture the error details without exposing them to the client
                        // Avoid leaking sensitive internal information in the response body for 5xx errors
                        match to_bytes(response.into_body(), usize::MAX).await {
                            Ok(content) if !content.is_empty() => tracing::error!("{:?}", content),
                            Err(e) => tracing::error!(
                                "unable to convert INTERNAL_SERVER_ERROR response body to bytes :: {:?}",
                                e
                            ),
                            _ => {}
                        }
                        status.into_response()
                    }
                }
            },
        ))
}
