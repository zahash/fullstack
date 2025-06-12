use std::sync::Arc;

use cache::{Cache, CacheRegistry, Tag};
use sqlx::SqlitePool;

pub struct DataAccess {
    pool: SqlitePool,
    cache_registry: Arc<CacheRegistry>,
}

impl DataAccess {
    pub fn new(pool: SqlitePool, cache_registry: CacheRegistry) -> Self {
        Self {
            pool,
            cache_registry: Arc::new(cache_registry),
        }
    }

    pub async fn read<'conn, K, V, Fut, C>(
        &'conn self,
        query: impl FnOnce(&'conn SqlitePool) -> Fut,
        namespace: &'static str,
        key: K,
        tagger: impl FnOnce(&V) -> Vec<Box<dyn Tag>>,
        cache_init: impl FnOnce() -> C,
    ) -> Fut::Output
    where
        K: 'static,
        V: Clone + 'static,
        Fut: Future<Output = Result<V, sqlx::Error>>,
        C: Cache<Key = K, Value = V> + Send + Sync + 'static,
    {
        self.cache_registry.ensure_cache(namespace, cache_init);
        match self.cache_registry.get::<K, V>(namespace, &key) {
            Some(value) => Ok(value),
            None => {
                let value = query(&self.pool).await?;
                self.cache_registry
                    .put(namespace, key, value.clone(), tagger(&value));
                Ok(value)
            }
        }
    }

    pub async fn write<'conn, V, Fut>(
        &'conn self,
        query: impl FnOnce(&'conn SqlitePool) -> Fut,
        tagger: impl FnOnce(&V) -> Vec<Box<dyn Tag>>,
    ) -> Fut::Output
    where
        Fut: Future<Output = Result<V, sqlx::Error>>,
        V: 'static,
    {
        let value = query(&self.pool).await?;
        for tag in tagger(&value) {
            self.cache_registry.invalidate(&*tag);
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
