use std::any::Any;

use crate::Cache;

pub trait CacheAny {
    fn get_any(&self, key: &dyn Any) -> Option<Box<dyn Any>>;
    fn put_any(&mut self, key: Box<dyn Any>, value: Box<dyn Any>, tags: Vec<Box<dyn Any>>);
    fn invalidate_any(&mut self, tag: &dyn Any);
}

impl<C> CacheAny for C
where
    C: Cache,
    C::Key: 'static,
    C::Value: 'static,
    C::Tag: 'static,
{
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?key), skip_all, ret)
    )]
    fn get_any(&self, key: &dyn Any) -> Option<Box<dyn Any>> {
        key.downcast_ref::<C::Key>()
            .or_else(|| {
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
    fn put_any(&mut self, key: Box<dyn Any>, value: Box<dyn Any>, tags: Vec<Box<dyn Any>>) {
        let key = key.downcast::<C::Key>().map(|b| *b).inspect_err(|_| {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                "failed to downcast key to {}",
                std::any::type_name::<C::Key>()
            );
        });

        let value = value.downcast::<C::Value>().map(|b| *b).inspect_err(|_| {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                "failed to downcast value to {}",
                std::any::type_name::<C::Value>()
            );
        });

        let tags = tags
            .into_iter()
            .map(|tag| {
                tag.downcast::<C::Tag>().map(|b| *b).inspect_err(|_| {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(
                        "failed to downcast tag to {}",
                        std::any::type_name::<C::Tag>()
                    );
                })
            })
            .collect();

        if let (Ok(k), Ok(v), Ok(tags)) = (key, value, tags) {
            self.put(k, v, tags);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", fields(?tag), skip_all)
    )]
    fn invalidate_any(&mut self, tag: &dyn Any) {
        if let Some(tag) = tag.downcast_ref::<C::Tag>().or_else(|| {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                "failed to downcast tag to {}",
                std::any::type_name::<C::Tag>()
            );
            None
        }) {
            self.invalidate(tag);
        }
    }
}
