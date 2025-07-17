use std::sync::Arc;

use cache::{Cache, CacheRegistry, CacheTypeConflictError};
use sqlx::SqlitePool;

pub struct DataAccess {
    pool: SqlitePool,
    cache_registry: Arc<CacheRegistry>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Cache(#[from] CacheTypeConflictError),

    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
}

impl DataAccess {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            cache_registry: Arc::new(CacheRegistry::new()),
        }
    }

    pub async fn read<
        'conn,
        #[cfg(not(feature = "tracing"))] K,
        #[cfg(not(feature = "tracing"))] V,
        #[cfg(not(feature = "tracing"))] T,
        #[cfg(feature = "tracing")] K: std::fmt::Debug,
        #[cfg(feature = "tracing")] V: std::fmt::Debug,
        #[cfg(feature = "tracing")] T: std::fmt::Debug,
        Fut,
        C,
    >(
        &'conn self,
        query: impl FnOnce(&'conn SqlitePool) -> Fut,
        namespace: &'static str,
        key: K,
        tags: impl FnOnce(&V) -> Vec<T>,
        cache_init: impl FnOnce() -> C,
    ) -> Result<V, Error>
    where
        K: 'static,
        V: Clone + 'static,
        T: 'static,
        Fut: Future<Output = Result<V, sqlx::Error>>,
        C: Cache<Key = K, Value = V, Tag = T> + Send + Sync + 'static,
    {
        self.cache_registry.ensure_cache(namespace, cache_init)?;
        match self.cache_registry.get::<K, V>(namespace, &key) {
            Some(value) => Ok(value),
            None => {
                let value = query(&self.pool).await?;
                self.cache_registry
                    .put(namespace, key, value.clone(), tags(&value));
                Ok(value)
            }
        }
    }

    pub async fn write<
        'conn,
        V,
        #[cfg(not(feature = "tracing"))] T,
        #[cfg(feature = "tracing")] T: std::fmt::Debug,
        Fut,
    >(
        &'conn self,
        query: impl FnOnce(&'conn SqlitePool) -> Fut,
        tags_to_invalidate: impl FnOnce(&V) -> Vec<T>,
    ) -> Result<V, Error>
    where
        Fut: Future<Output = Result<V, sqlx::Error>>,
        V: 'static,
        T: 'static,
    {
        let value = query(&self.pool).await?;
        for tag in tags_to_invalidate(&value) {
            self.cache_registry.invalidate(&tag);
        }
        Ok(value)
    }
}

impl Clone for DataAccess {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            cache_registry: Arc::clone(&self.cache_registry),
        }
    }
}
