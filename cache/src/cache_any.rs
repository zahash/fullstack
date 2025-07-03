use std::any::Any;

use crate::{Cache, Tag};

pub trait CacheAny {
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?key), skip_all, ret)
    )]
    fn get_any(&self, key: &dyn Any) -> Option<Box<dyn Any>> {
        key.downcast_ref::<C::Key>()
            .or({
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    "failed to downcast_ref key to {}",
                    std::any::type_name::<C::Key>()
                );

                None
            })
            .and_then(|k| self.get(k))
            .map(|v| Box::new(v) as Box<dyn Any>)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?key, ?value, ?tags), skip_all)
    )]
    fn put_any(&mut self, key: Box<dyn Any>, value: Box<dyn Any>, tags: Vec<Box<dyn Tag>>) {
        if let (Ok(k), Ok(v)) = (
            key.downcast::<C::Key>()
                .inspect_err(|_| {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(
                        "failed to downcast key to {}",
                        std::any::type_name::<C::Key>()
                    );
                })
                .map(|b| *b),
            value
                .downcast::<C::Value>()
                .inspect_err(|_| {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(
                        "failed to downcast value to {}",
                        std::any::type_name::<C::Value>()
                    );
                })
                .map(|b| *b),
        ) {
            self.put(k, v, tags);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?tag), skip_all)
    )]
    fn invalidate_any(&mut self, tag: &dyn Tag) {
        self.invalidate(tag);
    }
}
