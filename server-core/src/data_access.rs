use sqlx::SqlitePool;

use crate::cache::{CacheRegistry, Tag};

pub struct DataAccess {
    pool: SqlitePool,
    cache_registry: CacheRegistry,
}

impl DataAccess {
    pub async fn read<'conn, K, V, Fut>(
        &'conn self,
        query: impl FnOnce(&'conn SqlitePool) -> Fut,
        namespace: &'static str,
        key: K,
        tagger: impl FnOnce(&V) -> Vec<Box<dyn Tag>>,
    ) -> Fut::Output
    where
        Fut: Future<Output = Result<V, sqlx::Error>>,
        K: 'static,
        V: Clone + 'static,
    {
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

    pub async fn write<'conn, K, V, Fut>(
        &'conn self,
        query: impl FnOnce(&'conn SqlitePool) -> Fut,
        tagger: impl FnOnce(&V) -> Vec<Box<dyn Tag>>,
    ) -> Fut::Output
    where
        Fut: Future<Output = Result<V, sqlx::Error>>,
        K: 'static,
        V: Clone + 'static,
    {
        let value = query(&self.pool).await?;
        for tag in tagger(&value) {
            self.cache_registry.invalidate(&*tag);
        }
        Ok(value)
    }
}
