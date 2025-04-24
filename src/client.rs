use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{error::CacheError, options::CacheOptions};

#[derive(Debug, Clone)]
pub struct WorkerClient {
    client: Client,
    options: CacheOptions,
}

#[derive(Debug, Serialize)]
struct CacheRequest<T> {
    key: String,
    value: T,
    tags: Vec<String>,
    ttl: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CacheResponse {
    success: bool,
    data: Option<Value>,
    message: Option<String>,
}

impl WorkerClient {
    pub fn new(options: CacheOptions) -> Self {
        Self {
            client: Client::new(),
            options,
        }
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/get/{}", self.options.worker, key);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.options.api_key))
            .send()
            .await?;

        let cache_response: CacheResponse = response.json().await?;

        if !cache_response.success {
            if let Some(message) = cache_response.message {
                return Err(CacheError::Worker(message));
            }
            return Ok(None);
        }

        match cache_response.data {
            Some(data) => Ok(Some(serde_json::from_value(data)?)),
            None => Ok(None),
        }
    }

    pub async fn set<T>(&self, key: &str, value: T) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let url = format!("{}/set", self.options.worker);
        let ttl = self.options.revalidate.map(|d| d.as_millis() as u64);

        let req = CacheRequest {
            key: key.to_string(),
            value,
            tags: self.options.tags.clone(),
            ttl,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.options.api_key))
            .json(&req)
            .send()
            .await?;

        let cache_response: CacheResponse = response.json().await?;
        if !cache_response.success {
            if let Some(message) = cache_response.message {
                return Err(CacheError::Worker(message));
            }
            return Ok(());
        }

        Ok(())
    }

    pub async fn invalidate_tags(&self, tags: &[String]) -> Result<(), CacheError> {
        let url = format!("{}/invalidate", self.options.worker);
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.options.api_key))
            .json(&tags)
            .send()
            .await?;

        let cache_response: CacheResponse = response.json().await?;
        if !cache_response.success {
            if let Some(message) = cache_response.message {
                return Err(CacheError::Worker(message));
            }
            return Err(CacheError::Cache("Failed to Invalidate Cache".into()));
        }

        Ok(())
    }
}
