use std::hash::Hash;

use cache::Cache;
use dashmap::DashMap;

pub struct DashCache<K, V, T> {
    cache: DashMap<K, V>,
    tags: DashMap<T, Vec<K>>,
}

impl<
    #[cfg(not(feature = "tracing"))] K,
    #[cfg(not(feature = "tracing"))] V,
    #[cfg(not(feature = "tracing"))] T,
    #[cfg(feature = "tracing")] K: std::fmt::Debug,
    #[cfg(feature = "tracing")] V: std::fmt::Debug,
    #[cfg(feature = "tracing")] T: std::fmt::Debug,
> Cache for DashCache<K, V, T>
where
    K: Hash + Eq + Clone,
    V: Clone,
    T: Hash + Eq,
{
    type Key = K;
    type Value = V;
    type Tag = T;

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
    fn put(&mut self, key: Self::Key, value: Self::Value, tags: Vec<Self::Tag>) {
        self.cache.insert(key.clone(), value);
        for tag in tags {
            #[cfg(feature = "tracing")]
            tracing::trace!("inserting tag `{:?}`", tag);

            self.tags.entry(tag).or_default().push(key.clone());
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?tag), skip_all)
    )]
    fn invalidate(&mut self, tag: &Self::Tag) {
        if let Some((_, keys)) = self.tags.remove(tag) {
            for key in keys {
                #[cfg(feature = "tracing")]
                tracing::trace!("removing key `{:?}`", key);

                self.cache.remove(&key);
            }
        }
    }
}

impl<K, V, T> DashCache<K, V, T> {
    pub fn new() -> Self
    where
        K: Hash + Eq,
        T: Hash + Eq,
    {
        Self {
            cache: DashMap::new(),
            tags: DashMap::new(),
        }
    }
}

impl<K, V, T> Default for DashCache<K, V, T>
where
    K: Hash + Eq,
    T: Hash + Eq,
{
    fn default() -> Self {
        Self {
            cache: DashMap::new(),
            tags: DashMap::new(),
        }
    }
}
