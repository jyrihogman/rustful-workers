use std::future::Future;

use serde::{de::DeserializeOwned, Serialize};
use worker::kv::{KvError, KvStore};

pub fn get_cache<T: DeserializeOwned + 'static>(
    key: &str,
) -> Box<dyn Future<Output = Result<Option<T>, KvError>>> {
    let kv = Result::expect(KvStore::create("KvCache"), "KvCache store not found");

    Box::new(kv.get(key).cache_ttl(300).json::<T>())
}

pub fn set_cache<T: Serialize>(
    key: &str,
    value: T,
) -> Box<dyn Future<Output = Result<(), KvError>>> {
    let kv = Result::expect(KvStore::create("KvCache"), "KvCache store not found");

    let opts_builder = Result::expect(kv.put(key, value), "KV Insert failed");

    Box::new(async { opts_builder.execute().await })
}
