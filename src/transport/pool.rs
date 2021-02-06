use std::sync::Arc;
use std::hash::Hash;
use std::collections::{VecDeque, HashMap};
use tokio::sync::{Semaphore, Mutex};

pub trait Kind<T> {
    fn kind(&self) -> T;
}

/// The error returned on write to a closed pool
#[derive(Debug)]
pub struct WriteError<T>(pub T);

/// Asynchronous value pool
///
/// Can be used to exchange various kinds of data between asynchronous tasks
///
/// # Example
/// In this example, we are listening to different kinds of data from multiple tasks.
///
/// ```
/// use cobra_rs::transport::pool::Pool;
/// use cobra_rs::transport::pool::Kind;
/// use tokio::time::{sleep, Duration};
///
/// #[derive(Debug)]
/// struct Value {
///     kind: u8
/// }
///
/// impl Value {
///     fn create(kind: u8) -> Self {
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
/// const KIND_A: u8 = 1;
/// const KIND_B: u8 = 2;
///
/// #[tokio::main]
/// async fn main() {
///     let pool = Pool::new();
///     let pool2 = pool.clone();
///     let pool3 = pool.clone();
///
///     tokio::spawn(async move {
///         loop {
///             let data = pool2.read(KIND_A).await;
///             match data {
///                 Some(data) => println!("Received KIND_A value {:?}", data),
///                 None => break println!("Pool closed"),
///             }
///         }
///     });
///
///     tokio::spawn(async move {
///         loop {
///             let data = pool3.read(KIND_B).await;
///
///             match data {
///                 Some(data) => println!("Received KIND_B value {:?}", data),
///                 None => break println!("Pool closed"),
///             }
///         }
///     });
///
///     let value_a = Value::create(KIND_A);
///     let value_b = Value::create(KIND_B);
///
///     pool.write(value_a).await.unwrap();
///     sleep(Duration::from_secs(4)).await;
///
///     pool.write(value_b).await.unwrap();
///     sleep(Duration::from_secs(4)).await;
///
///     pool.close().await;
///     sleep(Duration::from_secs(1)).await;
/// }
/// ```
pub struct Pool<K: Eq + Hash, V: Kind<K>> {
    semaphore: Arc<Semaphore>,
    values: Arc<Mutex<KindStore<K, V>>>,
}

impl<K: Eq + Hash, V: Kind<K>> Pool<K, V> {
    /// Creates new pool
    pub fn new() -> Self {
        Default::default()
    }

    /// Reads value with specific kind from pool
    ///
    /// Returns [None] if the pool was closed
    ///
    /// # Note
    ///
    /// Do not read one kind of data from several tasks,
    /// this will lead to the selection of a random receiver
    ///
    /// [None]: std::option::Option
    pub async fn read(&self, kind: K) -> Option<V> {
        if let Some(value) = self.read_value(&kind).await {
            return Some(value);
        }

        loop {
            match self.semaphore.acquire().await {
                Ok(permit) => {
                    permit.forget();

                    let value = self.read_value(&kind).await;
                    if value.is_some() {
                        break value;
                    }
                }

                // Pool is closed
                Err(_) => break None,
            }
        }
    }

    async fn read_value(&self, kind: &K) -> Option<V> {
        self.values.lock().await.map.get_mut(kind)?.pop_front()
    }

    /// Writes value to the pool
    ///
    /// Returns [WriteError] if the pool was closed
    ///
    /// [WriteError]: crate::transport::pool::WriteError
    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
        {
            let mut value_store = self.values.lock().await;

            // If the pool was closed, we need to return the membership
            if !value_store.can_write {
                return Err(WriteError(value));
            }

            // Creating linked list if not exists and inserting frame
            value_store.map
                .entry(value.kind())
                .or_insert_with(VecDeque::new)
                .push_back(value);
        }

        self.semaphore.add_permits(1);
        Ok(())
    }

    /// Closes the pool
    pub async fn close(&self) {
        self.semaphore.close();
        self.values.lock().await.can_write = false;
    }
}

impl<K: Eq + Hash, V: Kind<K>> Clone for Pool<K, V> {
    fn clone(&self) -> Self {
        Pool {
            semaphore: self.semaphore.clone(),
            values: self.values.clone(),
        }
    }
}

impl<K: Eq + Hash, V: Kind<K>> Default for Pool<K, V> {
    fn default() -> Self {
        Pool {
            semaphore: Arc::new(Semaphore::new(0)),
            values: Arc::new(Mutex::new(KindStore::new())),
        }
    }
}

struct KindStore<K: Eq + Hash, V: Kind<K>> {
    map: HashMap<K, VecDeque<V>>,
    can_write: bool,
}

impl<K: Eq + Hash, V: Kind<K>> KindStore<K, V> {
    fn new() -> Self {
        Default::default()
    }
}

impl<K: Eq + Hash, V: Kind<K>> Default for KindStore<K, V> {
    fn default() -> Self {
        KindStore {
            map: HashMap::new(),
            can_write: true,
        }
    }
}

/// Asynchronous value pool without division into types
///
/// Same as [Pool], but does not separate values into different types
///
/// # Example
/// In this example, we are listening data from another task.
///
/// ```
/// use cobra_rs::transport::pool::PoolAny;
/// use tokio::time::{sleep, Duration};
///
/// #[derive(Debug)]
/// struct Value {
///     a: i32
/// }
///
/// impl Value {
///     fn create(a: i32) -> Self {
///         Value { a }
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let pool = PoolAny::new();
///     let pool2 = pool.clone();
///
///     tokio::spawn(async move {
///         loop {
///             let data = pool2.read().await;
///
///             match data {
///                 Some(data) => println!("Received value {:?}", data),
///                 None => {
///                     println!("Pool closed");
///                     break;
///                 }
///             }
///         }
///     });
///
///     let value_a = Value::create(1);
///     let value_b = Value::create(2);
///
///     pool.write(value_a).await.unwrap();
///     sleep(Duration::from_secs(4)).await;
///
///     pool.write(value_b).await.unwrap();
///     sleep(Duration::from_secs(4)).await;
///
///     pool.close().await;
///     sleep(Duration::from_secs(1)).await;
/// }
/// ```
///
/// [Pool]: crate::transport::pool::Pool
pub struct PoolAny<V> {
    semaphore: Arc<Semaphore>,
    values: Arc<Mutex<AnyStore<V>>>,
}

impl<V> PoolAny<V> {
    /// Creates new pool
    pub fn new() -> Self {
        Default::default()
    }

    /// Reads value written to the pool
    ///
    /// Returns [None] if the pool was closed
    ///
    /// [None]: sts::option::Option
    pub async fn read(&self) -> Option<V> {
        match self.semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();

                // Always Some()
                self.values.lock().await.deque.pop_front()
            }

            // Pool is closed
            Err(_) => None,
        }
    }

    /// Writes value to the pool
    ///
    /// Returns [WriteError] if the pool was closed
    ///
    /// [WriteError]: crate::transport::pool::WriteError
    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
        {
            let mut value_store = self.values.lock().await;

            // If the pool was closed, we need to return the membership
            if !value_store.can_write {
                return Err(WriteError(value));
            }

            value_store.deque.push_back(value);
        }
        self.semaphore.add_permits(1);
        Ok(())
    }

    /// Closes the pool
    pub async fn close(&self) {
        self.semaphore.close();
        self.values.lock().await.can_write = false;
    }
}

impl<V> Clone for PoolAny<V> {
    fn clone(&self) -> Self {
        PoolAny {
            semaphore: self.semaphore.clone(),
            values: self.values.clone(),
        }
    }
}

impl<V> Default for PoolAny<V> {
    fn default() -> Self {
        PoolAny {
            semaphore: Arc::new(Semaphore::new(0)),
            values: Arc::new(Mutex::new(AnyStore::new())),
        }
    }
}

struct AnyStore<V> {
    deque: VecDeque<V>,
    can_write: bool,
}

impl<V> AnyStore<V> {
    fn new() -> Self {
        Default::default()
    }
}

impl<V> Default for AnyStore<V> {
    fn default() -> Self {
        AnyStore {
            deque: VecDeque::new(),
            can_write: true,
        }
    }
}
