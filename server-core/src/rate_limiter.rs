use std::{
    collections::VecDeque,
    net::IpAddr,
    time::{Duration, Instant},
};

use dashmap::DashMap;

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
