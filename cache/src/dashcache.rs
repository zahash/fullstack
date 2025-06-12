use std::hash::Hash;

use dashmap::DashMap;

use crate::{Cache, Tag};

pub struct DashCache<K, V> {
    cache: DashMap<K, V>,
    tags: DashMap<String, Vec<K>>,
}

impl<K, V> Cache for DashCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    type Key = K;
    type Value = V;

    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.cache.get(key).map(|entry| entry.value().clone())
    }

    fn put(&mut self, key: Self::Key, value: Self::Value, tags: Vec<Box<dyn Tag>>) {
        self.cache.insert(key.clone(), value);
        for tag in tags {
            self.tags
                .entry(tag.id().to_string())
                .or_insert_with(Vec::new)
                .push(key.clone());
        }
    }

    fn invalidate(&mut self, tag: &dyn Tag) {
        if let Some((_, keys)) = self.tags.remove(tag.id()) {
            for key in keys {
                self.cache.remove(&key);
            }
        }
    }
}

impl<K, V> DashCache<K, V> {
    pub fn new() -> Self
    where
        K: Hash + Eq,
    {
        Self {
            cache: DashMap::new(),
            tags: DashMap::new(),
        }
    }
}
