#[cfg(feature = "tracing")]
pub trait Tag: std::fmt::Debug {
    fn id(&self) -> &str;
}

#[cfg(not(feature = "tracing"))]
pub trait Tag {
    fn id(&self) -> &str;
}

impl Tag for String {
    fn id(&self) -> &str {
        self.as_str()
    }
}

impl Tag for &str {
    fn id(&self) -> &str {
        self
    }
}
