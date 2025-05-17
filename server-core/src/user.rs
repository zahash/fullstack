use bcrypt::verify;
use sqlx::{Sqlite, SqlitePool, Type};

use crate::{Email, Username, Valid};

#[derive(Debug, Clone, PartialEq)]
pub struct UserId(i64);

#[derive(Debug)]
pub struct UserInfo {
    pub user_id: UserId,
    pub username: Username,
    pub email: Email,
    password_hash: String,
}

impl UserId {
    pub async fn info(&self, pool: &sqlx::SqlitePool) -> Result<Option<UserInfo>, sqlx::Error> {
        let record = sqlx::query!(
            r#"SELECT id as "user_id!", username, email, password_hash FROM users WHERE id = ?"#,
            self.0
        )
        .fetch_optional(pool)
        .await?;

        match record {
            Some(record) => Ok(Some(UserInfo {
                user_id: UserId::from(record.user_id),
                username: Username::try_from_sqlx(record.username)?,
                email: Email::try_from_sqlx(record.email)?,
                password_hash: record.password_hash,
            })),
            None => Ok(None),
        }
    }
}

impl UserInfo {
    pub async fn from_username(
        username: &str,
        pool: &sqlx::SqlitePool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let record = sqlx::query!(            
            r#"SELECT id as "user_id!", username, email, password_hash FROM users WHERE username = ?"#,
            username
        )
        .fetch_optional(pool)
        .await?;

        match record {
            Some(record) => {
                Ok(Some(UserInfo {
                    user_id: UserId::from(record.user_id),
                    username: Username::try_from_sqlx(record.username)?,
                    email: Email::try_from_sqlx(record.email)?,
                    password_hash: record.password_hash,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn verify_password(
        self,
        password: &str,
    ) -> Result<Option<Valid<UserInfo>>, bcrypt::BcryptError> {
        match verify(password, &self.password_hash) {
            Ok(true) => Ok(Some(Valid(self))),
            Ok(false) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl Valid<UserInfo> {
    pub async fn permissions(&self, pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
        let user_id = &self.0.user_id;

        let permissions = sqlx::query_scalar!(
            r#"SELECT p.permission FROM permissions p
              INNER JOIN user_permissions up ON up.permission_id = p.id
              WHERE up.user_id = ?"#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(permissions)
    }
}

impl Type<Sqlite> for UserId {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, Sqlite> for UserId {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as sqlx::Encode<Sqlite>>::encode_by_ref(&self.0, buf)
    }
}

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        UserId(value)
    }
}
