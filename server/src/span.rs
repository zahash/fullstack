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

    // We deliberately use `error_span!` (instead of `info_span!`) here to ensure that
    // this span is *always created* and *visible* even when the log level is set to `warn` or `error`.
    //
    // This guarantees that if an `error!` or `warn!` is emitted deeper in the request pipeline,
    // it will still inherit this span — and we’ll retain valuable context like:
    // - request ID
    // - client IP
    // - method + URI
    //
    // Yes, `error_span!` implies a high severity level, but here it's used strategically
    // to preserve structured logging in production environments where higher log levels are enforced.
    tracing::error_span!(
        "request",
        "{} {} {} {}",
        OptionDisplay(client_ip, "<unknown-client-ip>"),
        request_id,
        request.method(),
        request.uri(),
    )
}
