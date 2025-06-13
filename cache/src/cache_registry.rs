use std::any::Any;

use dashmap::DashMap;

use crate::{Cache, Tag, cache_any::CacheAny, cfg_debug::CfgDebug};

pub struct CacheRegistry {
    caches: DashMap<&'static str, Box<dyn CacheAny + Send + Sync>>,
}

impl CacheRegistry {
    pub fn new() -> Self {
        Self {
            caches: DashMap::new(),
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?namespace), skip_all)
    )]
    pub fn ensure_cache<C>(&self, namespace: &'static str, cache_init: impl FnOnce() -> C)
    where
        C: Cache + Send + Sync + 'static,
    {
        match self.caches.entry(namespace) {
            dashmap::Entry::Occupied(_) => {
                #[cfg(feature = "tracing")]
                tracing::debug!("cache already exists");
            }
            dashmap::Entry::Vacant(entry) => {
                entry.insert(Box::new(cache_init()));

                #[cfg(feature = "tracing")]
                tracing::debug!("new cache initialized");
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?namespace, ?key), skip_all, ret)
    )]
    pub fn get<K, V>(&self, namespace: &'static str, key: &K) -> Option<V>
    where
        K: 'static + CfgDebug,
        V: 'static + CfgDebug,
    {
        self.caches
            .get(namespace)
            .or_else(|| {
                #[cfg(feature = "tracing")]
                tracing::debug!("namespace not found");

                None
            })?
            .get_any(key as &dyn Any)
            .or_else(|| {
                #[cfg(feature = "tracing")]
                tracing::debug!("key not found");

                None
            })?
            .downcast::<V>()
            .inspect_err(|_| {
                #[cfg(feature = "tracing")]
                tracing::debug!("failed to downcast value to {}", std::any::type_name::<V>());
            })
            .ok()
            .map(|v| *v)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?namespace, ?key, ?value, ?tags), skip_all, ret)
    )]
    pub fn put<K, V>(&self, namespace: &str, key: K, value: V, tags: Vec<Box<dyn Tag>>) -> bool
    where
        K: 'static + CfgDebug,
        V: 'static + CfgDebug,
    {
        match self.caches.get_mut(namespace) {
            Some(mut cache) => {
                cache.put_any(Box::new(key), Box::new(value), tags);
                true
            }
            None => {
                #[cfg(feature = "tracing")]
                tracing::debug!("namespace not found");

                false
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?tag), skip_all)
    )]
    pub fn invalidate(&self, tag: &dyn Tag) {
        for mut ref_ in self.caches.iter_mut() {
            ref_.value_mut().invalidate_any(tag);
        }
    }
}
