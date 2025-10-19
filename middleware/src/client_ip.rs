use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, header::FORWARDED},
    middleware::Next,
};
use forwarded_header_value::{ForwardedHeaderValue, Identifier};

pub async fn mw_client_ip(mut request: Request<Body>, next: Next) -> Response<Body> {
    let ip = client_ip(&request);
    request.extensions_mut().insert(ip);
    next.run(request).await
}

fn client_ip<B>(request: &Request<B>) -> Option<IpAddr> {
    request
        .headers()
        .get(FORWARDED)
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
