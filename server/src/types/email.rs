use std::{str::FromStr, sync::LazyLock};

use compiletime_regex::regex;
use regex::Regex;
use serde::Deserialize;
use sqlx::{Sqlite, Type};

#[derive(Debug)]
pub struct Email(String);

impl FromStr for Email {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Email::try_from(s.to_string())
    }
}

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
const RE_EMAIL: LazyLock<Regex> = LazyLock::new(|| {
    regex!(
        r#"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"#
    )
});

impl TryFrom<String> for Email {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match RE_EMAIL.is_match(&value) {
            true => Ok(Self(value)),
            false => Err("email address must be of valid format as defined by the HTML5 Specification https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address"),
        }
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Self>()
            .map_err(|err| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &err))
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
        <String as sqlx::Encode<Sqlite>>::encode_by_ref(&self.0, buf)
    }
}
