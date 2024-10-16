use std::{
    convert::Infallible,
    fmt::{Debug, Display},
    ops::{ControlFlow, FromResidual, Try},
};

use serde::Deserialize;
use sqlx::{Sqlite, Type};

pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    pub fn reveal(self) -> T {
        self.0
    }

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

// impl<T: Serialize> Serialize for Redacted<T> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         serializer.serialize_str(&redacted::<T>())
//     }
// }

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

pub trait RedactedMap<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> Redacted<U>;
}

impl<'a, T> RedactedMap<&'a T> for &'a Redacted<T> {
    fn map<U>(self, f: impl FnOnce(&'a T) -> U) -> Redacted<U> {
        Redacted::from(f(self.reveal_ref()))
    }
}

impl<'a, T0, T1> RedactedMap<(&'a T0, &'a T1)> for (&'a Redacted<T0>, &'a Redacted<T1>) {
    fn map<U>(self, f: impl FnOnce((&'a T0, &'a T1)) -> U) -> Redacted<U> {
        let (r1, r2) = self;
        Redacted::from(f((r1.reveal_ref(), r2.reveal_ref())))
    }
}

impl<T, E> Try for Redacted<Result<T, E>> {
    type Output = Redacted<T>;
    type Residual = Result<Infallible, E>;

    fn from_output(output: Self::Output) -> Self {
        Redacted(Ok(output.0))
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            Ok(value) => ControlFlow::Continue(Redacted(value)),
            Err(err) => ControlFlow::Break(Err(err)),
        }
    }
}

impl<T, E> FromResidual<Result<Infallible, E>> for Redacted<Result<T, E>> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        Redacted(residual.map(|v| match v {}))
    }
}
