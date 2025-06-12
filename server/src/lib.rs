use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use cache::CacheRegistry;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use api::server;
use server_core::{AppState, DataAccess, RateLimiter};

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

pub async fn serve(opts: ServerOpts) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("{:?}", opts);

    let state = AppState {
        data_access: DataAccess::new(
            SqlitePool::connect(&opts.database_url)
                .await
                .with_context(|| format!("connect database :: {}", opts.database_url))?,
            CacheRegistry::new(),
        ),
        rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
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
