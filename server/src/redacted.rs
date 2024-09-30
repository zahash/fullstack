use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Type};

pub struct Redacted<T>(T);

impl<T: Clone> Redacted<T> {
    pub fn reveal(&self) -> T {
        self.0.clone()
    }
}

impl<T> Redacted<T> {
    pub fn reveal_ref(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Redacted<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Debug for Redacted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", redacted::<T>())
    }
}

impl<T> Display for Redacted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", redacted::<T>())
    }
}

impl<T: Serialize> Serialize for Redacted<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&redacted::<T>())
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Redacted<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer)?))
    }
}

impl<'q, T: sqlx::Encode<'q, Sqlite>> sqlx::Encode<'q, Sqlite> for Redacted<T> {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        (&self.0).encode(buf)
    }
}

impl<T: Type<Sqlite>> Type<Sqlite> for Redacted<T> {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        T::type_info()
    }
}

fn redacted<T>() -> String {
    format!("<REDACTED {}>", std::any::type_name::<T>())
}
