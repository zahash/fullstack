use std::any::Any;

use dashmap::DashMap;

pub trait Tag {
    fn id(&self) -> &str;
}

impl Tag for String {
    fn id(&self) -> &str {
        self.as_str()
    }
}

impl Tag for &str {
    fn id(&self) -> &str {
        self
    }
}

pub trait Cache {
    type Key;
    type Value;

    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn put(&mut self, key: Self::Key, value: Self::Value, tags: Vec<Box<dyn Tag>>);
    fn invalidate(&mut self, tag: &dyn Tag);
}

pub struct CacheRegistry {
    caches: DashMap<&'static str, Box<dyn CacheAny + Send + Sync>>,
}

impl CacheRegistry {
    pub fn new() -> Self {
        Self {
            caches: DashMap::new(),
        }
    }

    pub fn ensure_cache<C>(&self, namespace: &'static str, cache_init: impl FnOnce() -> C)
    where
        C: Cache + Send + Sync + 'static,
    {
        if let dashmap::Entry::Vacant(entry) = self.caches.entry(namespace) {
            entry.insert(Box::new(cache_init()));
        }
    }

    pub fn get<K, V>(&self, namespace: &'static str, key: &K) -> Option<V>
    where
        K: 'static,
        V: 'static,
    {
        self.caches
            .get(namespace)?
            .get_any(key as &dyn Any)?
            .downcast::<V>()
            .ok()
            .map(|v| *v)
    }

    pub fn put<K, V>(&self, namespace: &str, key: K, value: V, tags: Vec<Box<dyn Tag>>) -> bool
    where
        K: 'static,
        V: 'static,
    {
        match self.caches.get_mut(namespace) {
            Some(mut cache) => {
                cache.put_any(Box::new(key), Box::new(value), tags);
                true
            }
            None => false,
        }
    }

    pub fn invalidate(&self, tag: &dyn Tag) {
        for mut ref_ in self.caches.iter_mut() {
            ref_.value_mut().invalidate_any(tag);
        }
    }
}

trait CacheAny {
    fn get_any(&self, key: &dyn Any) -> Option<Box<dyn Any>>;
    fn put_any(&mut self, key: Box<dyn Any>, value: Box<dyn Any>, tags: Vec<Box<dyn Tag>>);
    fn invalidate_any(&mut self, tag: &dyn Tag);
}

impl<C> CacheAny for C
where
    C: Cache,
    C::Key: 'static,
    C::Value: 'static,
{
    fn get_any(&self, key: &dyn Any) -> Option<Box<dyn Any>> {
        key.downcast_ref::<C::Key>()
            .and_then(|k| self.get(k))
            .map(|v| Box::new(v) as Box<dyn Any>)
    }

    fn put_any(&mut self, key: Box<dyn Any>, value: Box<dyn Any>, tags: Vec<Box<dyn Tag>>) {
        if let (Ok(k), Ok(v)) = (
            key.downcast::<C::Key>().map(|b| *b),
            value.downcast::<C::Value>().map(|b| *b),
        ) {
            self.put(k, v, tags);
        }
    }

    fn invalidate_any(&mut self, tag: &dyn Tag) {
        self.invalidate(tag);
    }
}
