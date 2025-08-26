use std::{fmt::Display, str::FromStr, string::FromUtf8Error};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Email(lettre::Address);

const MSG: &str = "email must conform to the HTML5 Specification https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address";

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

impl From<Email> for lettre::Address {
    fn from(email: Email) -> Self {
        email.0
    }
}

impl AsRef<[u8]> for Email {
    fn as_ref(&self) -> &[u8] {
        let str: &str = self.0.as_ref();
        str.as_bytes()
    }
}

impl TryFrom<Vec<u8>> for Email {
    type Error = ParseError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let s = String::from_utf8(bytes)?;
        Self::try_from(s).map_err(ParseError::InvalidFormat)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("{0}")]
    InvalidFormat(&'static str),
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<lettre::Address>()
            .map(Email)
            .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &MSG))
    }
}

#[cfg(feature = "sqlx")]
impl Email {
    pub fn try_from_sqlx(value: String) -> Result<Self, sqlx::Error> {
        Self::from_str(&value).map_err(|e| {
            sqlx::Error::Decode(format!("invalid email in database :: {value} :: {e}").into())
        })
    }
}

#[cfg(feature = "sqlite")]
impl sqlx::Type<sqlx::Sqlite> for Email {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

#[cfg(feature = "sqlite")]
impl sqlx::Encode<'_, sqlx::Sqlite> for Email {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<sqlx::Sqlite>>::encode_by_ref(&self.0.to_string(), buf)
    }
}

#[cfg(feature = "sqlite")]
impl sqlx::Decode<'_, sqlx::Sqlite> for Email {
    fn decode(
        value: <sqlx::Sqlite as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Self::try_from(value).map_err(|err| err.into())
    }
}
