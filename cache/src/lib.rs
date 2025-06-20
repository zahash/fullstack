mod cache;
mod cache_any;
mod cache_registry;
mod tag;

pub use cache::Cache;
pub use cache_registry::CacheRegistry;
pub use tag::Tag;

#[cfg(feature = "dashcache")]
mod dashcache;
#[cfg(feature = "dashcache")]
pub use dashcache::DashCache;
