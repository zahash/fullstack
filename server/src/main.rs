use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::Context;
use axum::Router;
use sqlx::SqlitePool;
use tokio::net::TcpListener;

use server::{server, ui, AppState, RateLimiter};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL")?;
    tracing::info!(DATABASE_URL = %database_url);
    let port: u16 = std::env::var("PORT")
        .context("PORT")?
        .parse()
        .context("parse PORT")?;
    let ui_dir = std::env::var("UI").context("UI")?;
    tracing::info!(UI = %ui_dir);

    let state = AppState {
        pool: SqlitePool::connect(&database_url)
            .await
            .context(format!("connect database :: {}", database_url))?,
        rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
    };

    let ui = ui(&ui_dir);
    let server = server(state);
    let app = Router::new()
        .merge(ui)
        .merge(server)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)
        .await
        .context(format!("bind :: {}", addr))?;
    tracing::info!(
        "listening on {}",
        listener.local_addr().context("local_addr")?
    );
    axum::serve(listener, app).await.context("axum::serve")?;
    Ok(())
}
