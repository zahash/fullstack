use std::hash::Hash;

use dashmap::DashMap;

use crate::{Cache, Tag, cfg_debug::CfgDebug};

pub struct DashCache<K, V> {
    cache: DashMap<K, V>,
    tags: DashMap<String, Vec<K>>,
}

impl<K, V> Cache for DashCache<K, V>
where
    K: Hash + Eq + Clone + CfgDebug,
    V: Clone + CfgDebug,
{
    type Key = K;
    type Value = V;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?key), skip_all, ret)
    )]
    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.cache.get(key).map(|entry| entry.value().clone())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?key, ?value, ?tags), skip_all)
    )]
    fn put(&mut self, key: Self::Key, value: Self::Value, tags: Vec<Box<dyn Tag>>) {
        self.cache.insert(key.clone(), value);
        for tag in tags {
            #[cfg(feature = "tracing")]
            tracing::trace!("inserting tag `{:?}`", tag);

            self.tags
                .entry(tag.id().to_string())
                .or_insert_with(Vec::new)
                .push(key.clone());
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?tag), skip_all)
    )]
    fn invalidate(&mut self, tag: &dyn Tag) {
        if let Some((_, keys)) = self.tags.remove(tag.id()) {
            for key in keys {
                #[cfg(feature = "tracing")]
                tracing::trace!("removing key `{:?}`", key);

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
