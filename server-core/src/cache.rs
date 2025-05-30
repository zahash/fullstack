use std::{any::Any, collections::HashMap};

pub trait Tag {
    fn id(&self) -> &str;
}

pub trait Cache {
    type Key;
    type Value;

    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn put(&mut self, key: Self::Key, value: Self::Value, tags: Vec<Box<dyn Tag>>);
    fn invalidate(&mut self, tag: &dyn Tag);
}

pub struct CacheRegistry {
    caches: HashMap<&'static str, Box<dyn CacheAny>>,
}

impl CacheRegistry {
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
        }
    }

    pub fn builder() -> CacheRegistryBuilder {
        CacheRegistryBuilder {
            caches: HashMap::new(),
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

    pub fn put<K, V>(&mut self, namespace: &str, key: K, value: V, tags: Vec<Box<dyn Tag>>) -> bool
    where
        K: 'static,
        V: 'static,
    {
        match self.caches.get_mut(namespace) {
            Some(cache) => {
                cache.put_any(Box::new(key), Box::new(value), tags);
                true
            }
            None => false,
        }
    }

    pub fn invalidate(&mut self, tag: &dyn Tag) {
        for cache in self.caches.values_mut() {
            cache.invalidate_any(tag);
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

pub struct CacheRegistryBuilder {
    caches: HashMap<&'static str, Box<dyn CacheAny>>,
}

impl CacheRegistryBuilder {
    pub fn add<C>(mut self, namespace: &'static str, cache: C) -> Self
    where
        C: Cache + 'static,
    {
        self.caches.insert(namespace, Box::new(cache));
        self
    }

    pub fn build(self) -> CacheRegistry {
        CacheRegistry {
            caches: self.caches,
        }
    }
}
