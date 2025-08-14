pub mod check_availability;

pub async fn exists(pool: &sqlx::Pool<sqlx::Sqlite>, username: &str) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT id as "user_id!" FROM users WHERE username = ? LIMIT 1"#,
        username
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
