use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use anyhow::{anyhow, Context};
use axum::Router;
use lettre::SmtpTransport;
use sqlx::SqlitePool;
use tokio::net::TcpListener;

use server::{server, ui, AppState, RateLimiter};

#[derive(Debug)]
struct Config {
    database_url: String,
    port: u16,
    ui_dir: String,
    smtp: SMTPConfig,
}

#[derive(Debug)]
struct SMTPConfig {
    relay: String,
    username: String,
    password: String,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: Self::env_var("DATABASE_URL")?,
            port: Self::parse_env_var("PORT")?,
            ui_dir: Self::env_var("UI")?,
            smtp: SMTPConfig {
                relay: Self::env_var("SMTP_RELAY")?,
                username: Self::env_var("SMTP_USERNAME")?,
                password: Self::env_var("SMTP_PASSWORD")?,
            },
        })
    }

    fn parse_env_var<T: FromStr<Err: 'static + Send + Sync + std::error::Error>>(
        name: &str,
    ) -> anyhow::Result<T> {
        Self::env_var(name)?.parse::<T>().with_context(|| {
            format!(
                "cannot parse env var `{}` as {}",
                name,
                std::any::type_name::<T>()
            )
        })
    }

    fn env_var(name: &str) -> anyhow::Result<String> {
        Self::opt_env_var(name)?.with_context(|| format!("env var `{}` not found", name))
    }

    fn opt_env_var(name: &str) -> anyhow::Result<Option<String>> {
        std::env::var_os(name)
            .map(|v| {
                v.into_string()
                    .map_err(|_| anyhow!("env var `{}` not UTF-8", name))
            })
            .transpose()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let config = Config::from_env()?;
    tracing::info!(?config);

    let state = AppState {
        pool: SqlitePool::connect(&config.database_url)
            .await
            .context(format!("connect database :: {}", config.database_url))?,
        rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
        mailer: Arc::new(
            SmtpTransport::relay(&config.smtp.relay)?
                .credentials((config.smtp.username, config.smtp.password).into())
                .build(),
        ),
    };

    let ui = ui(&config.ui_dir);
    let server = server(state);
    let app = Router::new()
        .merge(ui)
        .merge(server)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
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
