use serde::{de, Deserialize, Serialize};
use worker::kv::KvStore;

use crate::error::StorageError;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub date: String,
    pub message: String,
}

fn get_kv_storage(binding: &str) -> Result<KvStore, StorageError> {
    KvStore::create(binding).map_err(|_| StorageError::KvStorageNotFound)
}

pub async fn get_json<T: de::DeserializeOwned>(
    namespace: &str,
    key: &str,
    cache_ttl: Option<u64>,
) -> Option<T> {
    let store = get_kv_storage(namespace).ok()?;

    store
        .get(key)
        .cache_ttl(cache_ttl.unwrap_or(60))
        .json::<T>()
        .await
        .ok()?
}

pub async fn get_messages_with_date(date_string: String) -> Result<Vec<Message>, StorageError> {
    let store = get_kv_storage("messages")?;
    let messages = store
        .get(&format!("messages-{}", date_string))
        .json::<Vec<Message>>()
        .await
        .unwrap();

    match messages {
        Some(m) => Ok(m),
        None => Ok(vec![]),
    }
}
