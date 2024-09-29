use std::fmt::Display;

use uuid::Uuid;

pub struct UserId(pub i64);

#[derive(Debug, Clone)]
pub struct TraceId(pub String);

impl TraceId {
    pub fn new() -> Self {
        TraceId(Uuid::new_v4().to_string())
    }

    // pub fn as_str(&self) -> &str {
    //     &self.0
    // }
}

impl Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
