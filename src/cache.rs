use crate::{client::WorkerClient, error::CacheError, options::CacheOptions};
use futures::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Cache {
    client: WorkerClient,
}

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
    ) -> impl Fn(Args) -> BoxFuture<'static, Result<R, CacheError>>
    where
        F: Fn(Args) -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
        Args: Clone + Serialize + Send + 'static,
        R: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        let client = self.client.clone();
        let mut cache_options = options.unwrap_or_default();
        cache_options.tags = tags;

        move |args: Args| {
            let func = func.clone();
            let args_clone = args.clone();
            let client = client.clone();
            let options = cache_options.clone();

            Box::pin(async move {
                let mut hasher = DefaultHasher::new();
                let args_serialized =
                    serde_json::to_string(&args_clone).map_err(CacheError::Serialization)?;
                args_serialized.hash(&mut hasher);

                let key = format!("{:x}", hasher.finish());

                if let Some(cached) = client.get::<R>(&key).await? {
                    return Ok(cached);
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
