use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::sync::{Pool, PoolGuard, WriteError};

const KIND_HASHMAP_CAPACITY: usize = 5;

/// Trait used to split data into different types
pub trait Kind<T: Eq + Hash> {
    /// Returns value kind
    fn kind(&self) -> T;
}

/// Asynchronous typed value pool
///
/// Same as [`Pool`], but can separate values into different types
///
/// # Example
///
/// ```
/// use cobra_rs::sync::{KindPool, Kind};
///
/// #[derive(Debug)]
/// struct Value {
///     kind: u8,
/// }
///
/// impl Value {
///     fn new(kind: u8) -> Self {
///         Value { kind }
///     }
/// }
///
/// impl Kind<u8> for Value {
///     fn kind(&self) -> u8 {
///         self.kind
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let pool = KindPool::new();
///     let pool2: KindPool<u8, Value> = pool.clone();
///     let pool3: KindPool<u8, Value> = pool.clone();
///
///     const KIND_1: u8 = 1;
///     const KIND_2: u8 = 2;
///
///     tokio::spawn(async move {
///         loop {
///             match pool2.read(KIND_1).await {
///                 Some(value) => {
///                     println!("Received value with KIND_1: {:?}", *value);
///                     value.accept();
///                 }
///                 None => break println!("Pool closed"),
///             }
///         }
///     });
///
///     tokio::spawn(async move {
///         loop {
///             match pool3.read(KIND_2).await {
///                 Some(value) => {
///                     println!("Received value with KIND_2: {:?}", *value);
///                     value.accept();
///                 }
///                 None => break println!("Pool closed"),
///             }
///         }
///     });
///
///     pool.write(Value::new(KIND_1)).await.unwrap();
///     pool.write(Value::new(KIND_2)).await.unwrap();
///     pool.close().await;
/// }
/// ```
pub struct KindPool<K: Eq + Hash, V: Kind<K>> {
    state: Arc<KindPoolState<K, V>>
}

struct KindPoolState<K: Eq + Hash, V: Kind<K>> {
    pools: RwLock<HashMap<K, Pool<V>>>,
    closed: RwLock<bool>,
}

impl<K: Eq + Hash, V: Kind<K>> KindPool<K, V> {
    /// Creates new kind pool
    pub fn new() -> Self {
        Default::default()
    }

    /// Writes value to the pool
    ///
    /// Unlocks if the reader of **the same type** has accepted or rejected the value.
    /// Returns [`WriteError`] if the value was rejected by another side or
    /// the pool was closed.
    ///
    /// [`WriteError`]: crate::transport::sync::WriteError
    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
        if self.state.is_closed().await {
            Err(WriteError::Closed(value))
        } else {
            self.state.get_pool(value.kind()).await.write(value).await
        }
    }

    /// Reads value with **specified kind**
    ///
    /// Returns [`PoolGuard`], which can be used to accept or reject
    /// the value and [`None`] if the pool was closed
    ///
    /// # Note
    ///
    /// When [`PoolGuard`] returned by this method accepts or rejects
    /// a value, **it will only unlock writer with the same type**
    ///
    /// [`PoolGuard`]: crate::transport::sync::PoolGuard
    /// [`None`]: std::option::Option::None
    pub async fn read(&self, kind: K) -> Option<PoolGuard<V>> {
        if self.state.is_closed().await {
            None
        } else {
            self.state.get_pool(kind).await.read().await
        }
    }

    /// Closes the pool
    pub async fn close(&self) {
        self.state.close().await;
    }
}

impl<K: Eq + Hash, V: Kind<K>> KindPoolState<K, V> {
    fn new() -> Self {
        KindPoolState {
            pools: RwLock::new(HashMap::with_capacity(KIND_HASHMAP_CAPACITY)),
            closed: RwLock::new(false),
        }
    }

    async fn get_pool(&self, kind: K) -> Pool<V> {
        self.pools.write().await
            .entry(kind).or_insert_with(Pool::new)
            .clone()
    }

    async fn close(&self) {
        *self.closed.write().await = true;
        for (_, pool) in self.pools.read().await.iter() {
            pool.close();
        }
    }

    async fn is_closed(&self) -> bool {
        *self.closed.read().await
    }
}

impl<K: Eq + Hash, V: Kind<K>> Default for KindPool<K, V> {
    fn default() -> Self {
        KindPool {
            state: Arc::new(KindPoolState::new()),
        }
    }
}

impl<K: Eq + Hash, V: Kind<K>> Clone for KindPool<K, V> {
    fn clone(&self) -> Self {
        KindPool {
            state: self.state.clone(),
        }
    }
}
