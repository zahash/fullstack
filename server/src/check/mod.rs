mod access_token;
mod username_availability;

pub use access_token::access_token;
pub use username_availability::username_availability;

use sqlx::SqlitePool;

use crate::types::Username;

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
