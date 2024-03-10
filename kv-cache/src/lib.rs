use serde::{de::DeserializeOwned, Serialize};
use worker::{
    console_error, console_log,
    kv::{KvError, KvStore},
};

pub async fn get_cache<T>(key: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    match KvStore::create("KvCache") {
        Ok(kv) => {
            match kv.get(key).cache_ttl(300).json::<T>().await {
                Ok(result) => result,
                Err(_) => {
                    console_log!("CacheMiss");
                    None
                } // Error occurred, return None
            }
        }
        Err(_) => {
            console_error!("Failed to create KvStore");
            None
        }
    }
}

pub async fn set_cache<T: Serialize>(key: &str, value: T) -> Result<(), KvError> {
    let kv = KvStore::create("KvCache")?;
    kv.put(key, value)?.execute().await
}
