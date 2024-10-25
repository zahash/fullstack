use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use serde::Deserialize;
use sqlx::{Sqlite, Type};

#[derive(Debug)]
pub struct Email(lettre::Address);

impl Deref for Email {
    type Target = lettre::Address;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Email {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

const MSG: &'static str = "expected valid email format as specified by the HTML5 Specification https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address";
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

impl From<lettre::Address> for Email {
    fn from(value: lettre::Address) -> Self {
        Self(value)
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

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email({})", self.0)
    }
}
