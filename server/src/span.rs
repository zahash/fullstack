use http::Request;
use tracing::Span;

pub fn span<B>(request: &Request<B>) -> Span {
    let request_id = match request.headers().get("x-request-id") {
        None => "<unknown-request-id>",
        Some(header_value) => match header_value.to_str() {
            Ok(value) => value,
            Err(_) => {
                tracing::warn!("malformed request id :: {:?}", header_value);
                "<malformed-request-id>"
            }
        },
    };

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

    let span = tracing::error_span!(
        "request",
        %request_id,
        method = %request.method(),
        uri = %request.uri(),
        ip = tracing::field::Empty
    );

    #[cfg(feature = "client-ip")]
    {
        let client_ip = request
            .extensions()
            .get::<Option<std::net::IpAddr>>()
            .copied()
            .flatten();

        match client_ip {
            Some(ip_addr) => span.record("ip", tracing::field::display(ip_addr)),
            None => span.record("ip", "<unknown-ip>"),
        };
    }

    span
}
