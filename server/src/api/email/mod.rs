pub mod check_availability;

#[cfg(feature = "smtp")]
pub mod check_verification_token;

#[cfg(feature = "smtp")]
pub mod initiate_verification;

use email::Email;

pub async fn exists(pool: &sqlx::Pool<sqlx::Sqlite>, email: &Email) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT id as "user_id!" FROM users WHERE email = ? LIMIT 1"#,
        email
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
