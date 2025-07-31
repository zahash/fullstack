pub mod check_availability;

use dashcache::DashCache;
use data_access::DataAccess;

use tag::Tag;

pub async fn exists(
    data_access: &DataAccess,
    username: &str,
) -> Result<bool, data_access::Error> {
    #[derive(Debug, Clone)]
    struct Row {
        user_id: i64,
    }

    let row = data_access
        .read(
            |pool| {
                sqlx::query_as!(
                    Row,
                    r#"SELECT id as "user_id!" FROM users WHERE username = ? LIMIT 1"#,
                    username
                )
                .fetch_optional(pool)
            },
            "username_exists",
            username.to_string(),
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
