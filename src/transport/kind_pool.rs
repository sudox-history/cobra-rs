use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::transport::pool::{Pool, WriteError, PoolOutput};

pub trait Kind<T> {
    fn kind(&self) -> T;
}

pub struct KindPool<K: Eq + Hash + Clone, V: Kind<K>> {
    pools: Arc<RwLock<HashMap<K, Pool<V>>>>
}

impl<K: Eq + Hash + Clone, V: Kind<K>> KindPool<K, V> {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
        println!("Write 1");
        let pool = match self.pools.read().await.get(&value.kind()) {
            Some(pool) => {
                println!("Write 2");
                pool.clone()
            },
            None => {
                println!("Write 3");
                let x = self.pools
                    .write()
                    .await
                    .insert(value.kind(), Pool::new())
                    .unwrap();
                println!("Write 3.5");
                x
            }
        };
        println!("Write 4");
        let x = pool.write(value).await;
        println!("Write 5");
        x
    }

    pub async fn read(&self, kind: K) -> Option<PoolOutput<V>> {
        println!("Read 1");
        let pool = match self.pools.read().await.get(&kind) {
            Some(pool) => {
                println!("Read 2");
                pool.clone()
            },
            None => {
                println!("Read 3");
                let x = self.pools
                    .write()
                    .await
                    .insert(kind, Pool::new())
                    .unwrap();
                println!("Read 3.5");
                x
            }
        };
        println!("Read 4");
        let x = pool.read().await;
        println!("Read 5");
        x
    }


    pub async fn close(&self) {
        for (_, pool) in self.pools.read().await.iter() {
            pool.close();
        }
    }
}

impl<K: Eq + Hash + Clone, V: Kind<K>> Default for KindPool<K, V> {
    fn default() -> Self {
        KindPool {
            pools: Arc::new(RwLock::new(HashMap::new()))
        }
    }
}

impl<K: Eq + Hash + Clone, V: Kind<K>> Clone for KindPool<K, V> {
    fn clone(&self) -> Self {
        KindPool {
            pools: self.pools.clone()
        }
    }
}
