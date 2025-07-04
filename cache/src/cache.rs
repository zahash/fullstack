pub trait Cache {
    type Key;
    type Value;
    type Tag;

    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn put(&mut self, key: Self::Key, value: Self::Value, tags: Vec<Self::Tag>);
    fn invalidate(&mut self, tag: &Self::Tag);
}
