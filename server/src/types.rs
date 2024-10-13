use std::fmt::Display;

use serde::Serialize;
use uuid::Uuid;

pub struct UserId(pub i64);

#[derive(Debug, Clone, Serialize)]
pub struct RequestId(String);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn unknown() -> Self {
        Self("<unknown>".into())
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
