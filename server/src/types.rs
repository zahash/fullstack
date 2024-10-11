use std::fmt::Display;

use uuid::Uuid;

pub struct UserId(pub i64);

#[derive(Debug)]
pub struct TraceId(String);

impl TraceId {
    pub fn new() -> Self {
        TraceId(Uuid::new_v4().to_string())
    }
}

impl Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
