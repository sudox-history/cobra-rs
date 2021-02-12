use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::transport::sync::{Pool, PoolGuard, WriteError};

pub trait Kind<T: Eq + Hash> {
    fn kind(&self) -> T;
}

pub struct KindPool<K: Eq + Hash, V: Kind<K>> {
    pools: Arc<RwLock<HashMap<K, Pool<V>>>>,
    closed: Arc<RwLock<bool>>,
}

impl<K: Eq + Hash, V: Kind<K>> KindPool<K, V> {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
        if *self.closed.read().await {
            return Err(WriteError::Closed(value));
        }
        let pool = self.pools
            .write()
            .await
            .entry(value.kind())
            .or_insert_with(Pool::new)
            .clone();
        pool.write(value).await
    }

    pub async fn read(&self, kind: K) -> Option<PoolGuard<V>> {
        if *self.closed.read().await {
            return None;
        }
        let pool = self.pools
            .write()
            .await
            .entry(kind)
            .or_insert_with(Pool::new)
            .clone();
        pool.read().await
    }

    pub async fn close(&self) {
        *self.closed.write().await = true;
        for (_, pool) in self.pools.read().await.iter() {
            pool.close();
        }
    }
}

impl<K: Eq + Hash, V: Kind<K>> Default for KindPool<K, V> {
    fn default() -> Self {
        KindPool {
            pools: Arc::new(RwLock::new(HashMap::new())),
            closed: Arc::new(RwLock::new(false)),
        }
    }
}

impl<K: Eq + Hash, V: Kind<K>> Clone for KindPool<K, V> {
    fn clone(&self) -> Self {
        KindPool {
            pools: self.pools.clone(),
            closed: self.closed.clone(),
        }
    }
}
