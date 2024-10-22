use std::{fmt::Debug, str::FromStr};

use serde::Deserialize;

pub struct Password(String);

impl Password {
    fn is_strong(s: &str) -> bool {
        if s.len() < 8 {
            return false;
        }

        let mut has_lowercase = false;
        let mut has_uppercase = false;
        let mut has_digit = false;
        let mut has_special_char = false;

        for c in s.chars() {
            if !has_lowercase && c.is_lowercase() {
                has_lowercase = true;
            } else if !has_uppercase && c.is_uppercase() {
                has_uppercase = true;
            } else if !has_digit && c.is_digit(10) {
                has_digit = true;
            } else if !has_special_char && r#"!@#$%^&*()_-+={}[]|\:;"'<>,.?/~`"#.contains(c) {
                has_special_char = true;
            }
        }

        has_lowercase && has_uppercase && has_digit && has_special_char
    }
}

impl FromStr for Password {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Password::try_from(s.to_string())
    }
}

impl TryFrom<String> for Password {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match Self::is_strong(&value) {
            true => Ok(Self(value)),
            false => Err("password must be at least 8 characters long, and include a mix of uppercase and lowercase letters, digits, and special characters"),
        }
    }
}

impl AsRef<[u8]> for Password {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.parse::<Password>() {
            Ok(password) => Ok(password),
            Err(err) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &err,
            )),
        }
    }
}

impl Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Password(***)").finish()
    }
}
