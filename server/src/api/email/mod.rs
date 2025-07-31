pub mod check_availability;

#[cfg(feature = "smtp")]
pub mod check_verification_token;

#[cfg(feature = "smtp")]
pub mod initiate_verification;

use dashcache::DashCache;
use data_access::DataAccess;
use email::Email;
use tag::Tag;

pub async fn exists(data_access: &DataAccess, email: &Email) -> Result<bool, data_access::Error> {
    #[derive(Debug, Clone)]
    struct Row {
        user_id: i64,
    }

    let row = data_access
        .read(
            |pool| {
                sqlx::query_as!(
                    Row,
                    r#"SELECT id as "user_id!" FROM users WHERE email = ? LIMIT 1"#,
                    email
                )
                .fetch_optional(pool)
            },
            "email_exists",
            email.clone(),
            |value| match value {
                Some(row) => vec![Tag {
                    table: "users",
                    primary_key: Some(row.user_id),
                }],
                None => vec![Tag {
                    table: "users",
                    primary_key: None,
                }],
            },
            DashCache::new,
        )
        .await?;

    match row {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
