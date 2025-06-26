use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOptions {
    pub worker: String,
    pub api_key: String,
    #[serde(with = "duration_millis")]
    pub revalidate: Option<Duration>,
    pub tags: Vec<String>,
    pub should_cache: Option<bool>,
}

impl Default for CacheOptions {
    fn default() -> Self {
        CacheOptions {
            worker: String::new(),
            api_key: String::new(),
            revalidate: Some(Duration::from_secs(604800)), // 1 week
            tags: Vec::new(),
            should_cache: None,
        }
    }
}

mod duration_millis {
    use serde::{Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(dur: &Option<Duration>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dur {
            Some(d) => ser.serialize_u64(d.as_millis() as u64),
            None => ser.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis: Option<u64> = Option::deserialize(de)?;
        Ok(millis.map(Duration::from_millis))
    }
}
