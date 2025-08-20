pub trait ErrorKind {
    fn kind(&self) -> &'static str;
}
