mod cache;
mod client;
mod error;
mod options;

pub use cache::Cache;
pub use error::CacheError;
pub use options::CacheOptions;

pub fn create_cache(options: CacheOptions) -> Cache {
    Cache::new(options)
}
