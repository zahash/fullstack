use std::fmt::Display;

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct RequestId(Option<Uuid>);

impl RequestId {
    pub fn new() -> Self {
        Self(Some(Uuid::new_v4()))
    }

    pub fn unknown() -> Self {
        Self(None)
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(request_id) => write!(f, "{}", request_id),
            None => write!(f, "<unknown>"),
        }
    }
}
