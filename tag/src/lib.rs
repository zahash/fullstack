#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Tag {
    pub table: &'static str,
    pub primary_key: Option<i64>,
}
