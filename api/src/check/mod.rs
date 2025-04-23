mod access_token;
mod email_availability;
mod username_availability;

pub use access_token::access_token;
pub use email_availability::email_availability;
pub use username_availability::username_availability;

use sqlx::SqlitePool;

use server_core::{Email, Username};

pub async fn username_exists(pool: &SqlitePool, username: &Username) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = ? LIMIT 1) as username_exists",
        username
    )
    .fetch_one(pool)
    .await?
    .username_exists
    {
        0 => Ok(false),
        _ => Ok(true),
    }
}

pub async fn email_exists(pool: &SqlitePool, email: &Email) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = ? LIMIT 1) as email_exists",
        email
    )
    .fetch_one(pool)
    .await?
    .email_exists
    {
        0 => Ok(false),
        _ => Ok(true),
    }
}
