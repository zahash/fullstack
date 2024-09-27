mod error;
mod extractors;
mod login;
mod private;
mod signup;
mod types;

use std::net::SocketAddr;

use axum::{
    extract::Extension,
    http::{
        header::{CONTENT_TYPE, COOKIE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

use error::AppError;
use login::login;
use private::private;
use signup::signup;
use tower_http::cors::CorsLayer;

const FRONTEND_ORIGIN: HeaderValue = HeaderValue::from_static("http://127.0.0.1:3000");

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt().init();

    dotenv::from_filename(".env")?;
    let database_url = std::env::var("DATABASE_URL")?;
    let port: u16 = std::env::var("PORT")?.parse()?;

    let cors = CorsLayer::new()
        .allow_origin(FRONTEND_ORIGIN)
        .allow_credentials(true) // Allow cookies/sessions
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE, COOKIE]);

    let pool = SqlitePool::connect(&database_url).await?;

    let app = Router::new()
        .route("/", get(hello))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/private", get(private))
        .layer(Extension(pool))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn hello() -> &'static str {
    "hello world"
}
