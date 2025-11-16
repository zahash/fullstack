use bcrypt::verify;

use email::Email;

use crate::core::{Permission, Verified, permission::Authorizable};

pub struct UserInfo {
    pub user_id: i64,
    pub username: String,
    pub email: Email,
    password_hash: String,
}

impl UserInfo {
    pub async fn from_user_id(
        user_id: i64,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Option<UserInfo>, sqlx::Error> {
        #[derive(Debug, Clone)]
        struct Row {
            user_id: i64,
            username: String,
            email: String,
            password_hash: String,
        }

        let record = sqlx::query_as!(
            Row,
            r#"
            SELECT id as "user_id!", username, email, password_hash
            FROM users WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(pool)
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
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Option<Self>, sqlx::Error> {
        #[derive(Debug, Clone)]
        struct Row {
            user_id: i64,
            username: String,
            email: String,
            password_hash: String,
        }

        let record = sqlx::query_as!(
            Row,
            r#"
            SELECT id as "user_id!", username, email, password_hash
            FROM users WHERE username = ?
            "#,
            username
        )
        .fetch_optional(pool)
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

impl Authorizable for Verified<UserInfo> {
    async fn has_permission(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        let user_id = self.0.user_id;

        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM permissions p
                INNER JOIN user_permissions up ON up.permission_id = p.id
                WHERE up.user_id = ? AND p.permission = ?
            )
            "#,
            user_id,
            permission
        )
        .fetch_one(pool)
        .await?;

        Ok(exists != 0)
    }

    async fn permissions(
        &self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        let user_id = self.0.user_id;

        sqlx::query_as!(
            Permission,
            r#"
            SELECT p.id as "id!", p.permission, p.description FROM permissions p
            INNER JOIN user_permissions up ON up.permission_id = p.id
            WHERE up.user_id = ?
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
    }
}
