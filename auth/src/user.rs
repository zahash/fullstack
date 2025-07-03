use bcrypt::verify;
use cache::{DashCache, Tag};
use data_access::DataAccess;
use email::Email;

use crate::{Permission, Permissions, Verified};

pub struct UserInfo {
    pub user_id: i64,
    pub username: String,
    pub email: Email,
    password_hash: String,
}

impl UserInfo {
    pub async fn from_user_id(
        user_id: i64,
        data_access: &DataAccess,
    ) -> Result<Option<UserInfo>, sqlx::Error> {
        #[derive(Debug, Clone)]
        struct Row {
            user_id: i64,
            username: String,
            email: String,
            password_hash: String,
        }

        let record = data_access
            .read(
                |pool| {
                    sqlx::query_as!(
                        Row,
                        r#"
                        SELECT id as "user_id!", username, email, password_hash
                        FROM users WHERE id = ?
                        "#,
                        user_id
                    )
                    .fetch_optional(pool)
                },
                "user_info__from__user_id",
                user_id,
                |value| match value {
                    Some(row) => vec![Box::new(format!("users:{}", row.user_id))],
                    None => vec![Box::new("users")],
                },
                DashCache::new,
            )
            .await?;

        match record {
            Some(record) => Ok(Some(UserInfo {
                user_id: record.user_id,
                username: record.username,
                email: Email::try_from_sqlx(record.email)?,
                password_hash: record.password_hash,
            })),
            None => Ok(None),
        }
    }

    pub async fn from_username(
        username: &str,
        data_access: &DataAccess,
    ) -> Result<Option<Self>, sqlx::Error> {
        #[derive(Debug, Clone)]
        struct Row {
            user_id: i64,
            username: String,
            email: String,
            password_hash: String,
        }

        let record = data_access
            .read(
                |pool| {
                    sqlx::query_as!(
                        Row,
                        r#"
                        SELECT id as "user_id!", username, email, password_hash
                        FROM users WHERE username = ?
                        "#,
                        username
                    )
                    .fetch_optional(pool)
                },
                "user_info__from__username",
                username.to_string(),
                |value| match value {
                    Some(row) => vec![Box::new(format!("users:{}", row.user_id))],
                    None => vec![Box::new("users")],
                },
                DashCache::new,
            )
            .await?;

        match record {
            Some(record) => Ok(Some(UserInfo {
                user_id: record.user_id,
                username: record.username,
                email: Email::try_from_sqlx(record.email)?,
                password_hash: record.password_hash,
            })),
            None => Ok(None),
        }
    }

    pub fn verify_password(
        self,
        password: &str,
    ) -> Result<Option<Verified<UserInfo>>, bcrypt::BcryptError> {
        match verify(password, &self.password_hash) {
            Ok(true) => Ok(Some(Verified(self))),
            Ok(false) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl Verified<UserInfo> {
    pub async fn permissions(&self, data_access: &DataAccess) -> Result<Permissions, sqlx::Error> {
        let user_id = self.0.user_id;

        data_access
            .read(
                |pool| {
                    sqlx::query_as!(
                        Permission,
                        r#"
                        SELECT p.id as "id!", p.permission, p.description FROM permissions p
                        INNER JOIN user_permissions up ON up.permission_id = p.id
                        WHERE up.user_id = ?"#,
                        user_id
                    )
                    .fetch_all(pool)
                },
                "user_permissions__from__user_id",
                user_id,
                |permissions| {
                    let mut tags = permissions
                        .iter()
                        .map(|p| format!("permissions:{}", p.id))
                        .map(|tag| Box::new(tag) as Box<dyn Tag>)
                        .collect::<Vec<Box<dyn Tag + 'static>>>();
                    tags.push(Box::new(format!("users:{user_id}")));
                    tags
                },
                DashCache::new,
            )
            .await
            .map(Permissions)
    }
}
