use std::{fmt::Display, str::FromStr, sync::LazyLock};

use compiletime::regex;
use regex::Regex;
use serde::Deserialize;
use sqlx::{Sqlite, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(String);

impl FromStr for Username {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Username::try_from(s.to_string())
    }
}

const RE_USERNAME: LazyLock<Regex> = LazyLock::new(|| regex!(r#"^[A-Za-z0-9_]{2,30}$"#));

impl TryFrom<String> for Username {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match RE_USERNAME.is_match(&value) {
            true => Ok(Self(value)),
            false => Err("username must be between 2-30 in length. must only contain `A-Z` `a-z` `0-9` and `_`"),
        }
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

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Username({})", self.0)
    }
}
