pub mod check_availability;

#[cfg(feature = "smtp")]
pub mod verify_email;

#[cfg(feature = "smtp")]
pub mod initiate_verification;

use email::Email;
use sqlx::{Executor, Sqlite};

pub async fn exists<'a, E: Executor<'a, Database = Sqlite>>(
    ex: E,
    email: &Email,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT id as "user_id!" FROM users WHERE email = ? LIMIT 1"#,
        email
    )
    .fetch_optional(ex)
    .await?;

    match row {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
