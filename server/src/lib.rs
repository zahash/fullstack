mod access_token;
mod check;
mod error;
mod health;
mod login;
mod middleware;
mod misc;
mod private;
mod signup;
mod token;
mod types;

use std::{
    collections::VecDeque,
    fmt::Display,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use axum::{
    Router,
    extract::ConnectInfo,
    http::{Request, header},
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
};
use dashmap::DashMap;
use forwarded_header_value::{ForwardedHeaderValue, Identifier};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::Span;
// use lettre::SmtpTransport;
use middleware::{mw_client_ip, mw_handle_leaked_5xx, mw_rate_limiter};
use sqlx::SqlitePool;

use health::{health, sysinfo};
use login::login;
use private::private;
use signup::signup;

pub use types::{Email, Password, Username};

// TODO
// create permissions model
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

pub fn server(state: AppState) -> Router {
    Router::new()
        .nest(
            "/check",
            Router::new()
                .route("/username-availability", get(check::username_availability))
                .route("/email-availability", get(check::email_availability))
                .route("/access-token", get(check::access_token)),
        )
        .route("/health", get(health))
        .route("/sysinfo", get(sysinfo))
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
                .get::<ConnectInfo<SocketAddr>>()
                .map(|connect_info| connect_info.0.ip())
        })
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            rate_limiter: Arc::clone(&self.rate_limiter),
            // mailer: Arc::clone(&self.mailer),
        }
    }
}

#[derive(Debug)]
pub struct ServerOpts {
    pub database_url: String,
    pub port: u16,
    pub ui_dir: PathBuf,
    // pub smtp: SMTPConfig,
}

#[derive(Debug)]
pub struct SMTPConfig {
    pub relay: String,
    pub username: String,
    pub password: String,
}

pub async fn run(opts: ServerOpts) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("{:?}", opts);

    let state = AppState {
        pool: SqlitePool::connect(&opts.database_url)
            .await
            .with_context(|| format!("connect database :: {}", opts.database_url))?,
        rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
        // mailer: Arc::new(()),
        // mailer: Arc::new(
        //     SmtpTransport::relay(&opts.smtp.relay)?
        //         .credentials((opts.smtp.username, opts.smtp.password).into())
        //         .build(),
        // ),
    };

    let app = server(state)
        .fallback_service(ServeDir::new(&opts.ui_dir))
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind :: {}", addr))?;
    tracing::info!(
        "listening on {}",
        listener.local_addr().context("local_addr")?
    );
    axum::serve(listener, app).await.context("axum::serve")?;
    Ok(())
}
