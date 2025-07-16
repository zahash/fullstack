use std::any::{Any, TypeId};

use dashmap::DashMap;

use crate::{Cache, cache_any::CacheAny};

#[derive(thiserror::Error, Debug)]
#[error("cache namespace `{namespace}` type conflict: existing={existing:?}, new={new:?}")]
pub struct CacheTypeConflictError {
    pub namespace: &'static str,
    pub existing: TypeId,
    pub new: TypeId,
}

pub struct CacheRegistry {
    caches: DashMap<&'static str, (TypeId, Box<dyn CacheAny + Send + Sync>)>,
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
    pub fn ensure_cache<C>(
        &self,
        namespace: &'static str,
        cache_init: impl FnOnce() -> C,
    ) -> Result<(), CacheTypeConflictError>
    where
        C: Cache + Send + Sync + 'static,
    {
        let new_id = TypeId::of::<C>();

        match self.caches.entry(namespace) {
            dashmap::Entry::Occupied(entry) => {
                let (existing_id, _) = entry.get();

                match *existing_id == new_id {
                    true => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("cache already exists");

                        Ok(())
                    }
                    false => Err(CacheTypeConflictError {
                        namespace,
                        existing: *existing_id,
                        new: new_id,
                    }),
                }
            }
            dashmap::Entry::Vacant(entry) => {
                entry.insert((new_id, Box::new(cache_init())));

                #[cfg(feature = "tracing")]
                tracing::debug!("new cache initialized");

                Ok(())
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?namespace, ?key), skip_all, ret)
    )]
    pub fn get<
        #[cfg(not(feature = "tracing"))] K,
        #[cfg(not(feature = "tracing"))] V,
        #[cfg(feature = "tracing")] K: std::fmt::Debug,
        #[cfg(feature = "tracing")] V: std::fmt::Debug,
    >(
        &self,
        namespace: &'static str,
        key: &K,
    ) -> Option<V>
    where
        K: 'static,
        V: 'static,
    {
        self.caches
            .get(namespace)
            .or_else(|| {
                #[cfg(feature = "tracing")]
                tracing::debug!("namespace not found");

                None
            })?
            .1
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
    pub fn put<
        #[cfg(not(feature = "tracing"))] K,
        #[cfg(not(feature = "tracing"))] V,
        #[cfg(not(feature = "tracing"))] T,
        #[cfg(feature = "tracing")] K: std::fmt::Debug,
        #[cfg(feature = "tracing")] V: std::fmt::Debug,
        #[cfg(feature = "tracing")] T: std::fmt::Debug,
    >(
        &self,
        namespace: &str,
        key: K,
        value: V,
        tags: Vec<T>,
    ) -> bool
    where
        K: 'static,
        V: 'static,
        T: 'static,
    {
        match self.caches.get_mut(namespace) {
            Some(mut cache) => {
                cache.1.put_any(
                    Box::new(key),
                    Box::new(value),
                    tags.into_iter()
                        .map(|tag| Box::new(tag) as Box<dyn Any>)
                        .collect(),
                );
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
    pub fn invalidate<
        #[cfg(not(feature = "tracing"))] T,
        #[cfg(feature = "tracing")] T: std::fmt::Debug,
    >(
        &self,
        tag: &T,
    ) where
        T: 'static,
    {
        for mut ref_ in self.caches.iter_mut() {
            ref_.value_mut().1.invalidate_any(tag);
        }
    }
}

impl Default for CacheRegistry {
    fn default() -> Self {
        Self {
            caches: DashMap::new(),
        }
    }
}
