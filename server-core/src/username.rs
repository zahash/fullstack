use std::{fmt::Display, str::FromStr};

use serde::Deserialize;
use sqlx::{Sqlite, Type};
use validation::validate_username;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(String);

impl Username {
    pub fn try_from_sqlx(value: String) -> Result<Self, sqlx::Error> {
        Self::from_str(&value).map_err(|e| {
            sqlx::Error::Decode(
                format!("invalid username in database :: {} :: {}", value, e).into(),
            )
        })
    }
}

impl FromStr for Username {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_username(s.to_string()).map(Self)
    }
}

impl TryFrom<String> for Username {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        validate_username(value).map(Self)
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Self>()
            .map_err(|err| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &err))
    }
}

impl Type<Sqlite> for Username {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, Sqlite> for Username {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<Sqlite>>::encode_by_ref(&self.0, buf)
    }
}

impl sqlx::Decode<'_, Sqlite> for Username {
    fn decode(
        value: <Sqlite as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <String as sqlx::Decode<Sqlite>>::decode(value)?;
        Self::try_from(value).map_err(|err| err.into())
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Username({})", self.0)
    }
}
