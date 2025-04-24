use serde::{de::DeserializeOwned, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::future::Future;
use std::hash::{Hash, Hasher};

use crate::client::WorkerClient;
use crate::error::CacheError;
use crate::options::CacheOptions;

pub struct Cache {
    client: WorkerClient,
}

type CacheFuture<R> = Box<dyn Future<Output = Result<R, CacheError>> + Send + 'static>;
impl Cache {
    pub fn new(options: CacheOptions) -> Self {
        Self {
            client: WorkerClient::new(options),
        }
    }

    pub async fn invalidate_by_tag(&self, tags: Vec<String>) -> Result<(), CacheError> {
        self.client.invalidate_tags(&tags).await
    }

    pub fn cache<F, Fut, Args, R>(
        &self,
        func: F,
        tags: Vec<String>,
        options: Option<CacheOptions>,
    ) -> impl Fn(Args) -> CacheFuture<R>
    where
        F: Fn(Args) -> Fut + Clone + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
        Args: Clone + Serialize + Send + 'static,
        R: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        let client = self.client.clone();
        let mut cache_options = options.unwrap_or_else(|| CacheOptions::default());
        cache_options.tags = tags;

        move |args: Args| {
            let func = func.clone();
            let args_clone = args.clone();
            let client = client.clone();
            let options = cache_options.clone();

            Box::new(async move {
                let mut hasher = DefaultHasher::new();
                let args_serialized =
                    serde_json::to_string(&args_clone).map_err(CacheError::Serialization)?;
                args_serialized.hash(&mut hasher);

                let key = format!("{:x}", hasher.finish());
                if let Some(cached_result) = client.get::<R>(&key).await? {
                    return Ok(cached_result);
                }

                let result = func(args).await;
                let should_cache = options.should_cache.unwrap_or(true);
                if should_cache {
                    client.set(&key, &result).await?;
                }

                Ok(result)
            })
        }
    }
}
