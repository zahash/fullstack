use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use serde::Deserialize;
use sqlx::{Sqlite, Type};

#[derive(Debug)]
pub struct Email(lettre::Address);

const MSG: &'static str = "email must conform to the HTML5 Specification https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address";

impl Email {
    pub fn try_from_sqlx(value: String) -> Result<Self, sqlx::Error> {
        Self::from_str(&value).map_err(|e| {
            sqlx::Error::Decode(format!("invalid email in database :: {} :: {}", value, e).into())
        })
    }
}

impl FromStr for Email {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Email(lettre::Address::from_str(s).map_err(|_| MSG)?))
    }
}

impl TryFrom<String> for Email {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Email(lettre::Address::try_from(value).map_err(|_| MSG)?))
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<lettre::Address>()
            .map(|address| Email(address))
            .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &MSG))
    }
}

impl Type<Sqlite> for Email {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, Sqlite> for Email {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<Sqlite>>::encode_by_ref(&self.0.to_string(), buf)
    }
}

impl sqlx::Decode<'_, Sqlite> for Email {
    fn decode(
        value: <Sqlite as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <String as sqlx::Decode<Sqlite>>::decode(value)?;
        Self::try_from(value).map_err(|err| err.into())
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email({})", self.0)
    }
}
