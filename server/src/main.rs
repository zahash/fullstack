mod error;
mod login;
mod signup;

use std::net::SocketAddr;

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use error::AppError;
use login::login;
use signup::signup;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv::from_filename(".env")?;
    let database_url = std::env::var("DATABASE_URL")?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let pool = SqlitePool::connect(&database_url).await?;

    let app = Router::new()
        .route("/", get(hello))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .layer(Extension(pool));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn hello() -> &'static str {
    "hello world"
}
