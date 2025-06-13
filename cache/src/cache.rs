use crate::Tag;

pub trait Cache {
    type Key;
    type Value;

    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn put(&mut self, key: Self::Key, value: Self::Value, tags: Vec<Box<dyn Tag>>);
    fn invalidate(&mut self, tag: &dyn Tag);
}
