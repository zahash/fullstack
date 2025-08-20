use std::{
    collections::VecDeque,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response},
    middleware::Next,
    response::IntoResponse,
};
use axum_macros::debug_middleware;
use dashmap::DashMap;
use http::StatusCode;

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

    #[allow(dead_code)]
    pub fn nolimit() -> Self {
        Self {
            requests: DashMap::default(),
            limit: usize::MAX,
            interval: Duration::from_secs(0),
        }
    }

    pub fn is_too_many(&self, ip_addr: IpAddr) -> bool {
        let now = Instant::now();
        let mut request_timeline = self.requests.entry(ip_addr).or_default();

        // clean up old entries
        while let Some(time) = request_timeline.front() {
            if now.duration_since(*time) > self.interval {
                request_timeline.pop_front();
            } else {
                break;
            }
        }

        if request_timeline.len() >= self.limit {
            return true;
        }

        request_timeline.push_back(now);
        false
    }
}

#[debug_middleware]
pub async fn mw_rate_limiter(
    State(rate_limiter): State<Arc<RateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let client_ip = request
        .extensions()
        .get::<Option<IpAddr>>()
        .copied()
        .flatten()
        .unwrap_or_else(|| {
            #[cfg(feature = "tracing")]
            tracing::warn!("unable to get client_ip while rate limiting");

            IpAddr::from([0, 0, 0, 0])
        });

    if rate_limiter.is_too_many(client_ip) {
        #[cfg(feature = "tracing")]
        tracing::warn!("rate limited {}", client_ip);

        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }

    next.run(request).await
}
