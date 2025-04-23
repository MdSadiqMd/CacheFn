use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Worker returned error: {0}")]
    Worker(String),
}
