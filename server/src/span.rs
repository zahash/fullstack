use std::{fmt::Display, net::IpAddr};

use http::Request;
use tracing::Span;

struct OptionDisplay<T>(Option<T>, &'static str);

impl<T: Display> Display for OptionDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(val) => write!(f, "{val}"),
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
