mod cache;
mod cache_any;
mod cache_registry;

pub use cache::Cache;
pub use cache_registry::CacheRegistry;

#[cfg(feature = "dashcache")]
mod dashcache;
#[cfg(feature = "dashcache")]
pub use dashcache::DashCache;
