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
/// Can be used to exchange various kinds of data between asynchronous tasks
///
/// # Example
/// In this example, we are listening to different kinds of data from multiple tasks.
///
/// ```no_run
/// use cobra_rs::transport::pool::Pool;
/// use cobra_rs::transport::frame::Frame;
/// use tokio::time;
///
/// const PING_KIND: u8 = 1;
/// const HANDSHAKE_KIND: u8 = 2;
///
/// #[tokio::main]
/// async fn main() {
///     let pool = Pool::new();
///     let pool2 = pool.clone();
///     let pool3 = pool.clone();
///
///     tokio::spawn(async move {
///         loop {
///             let data = pool2.read(PING_KIND).await;
///             match data {
///                 Some(data) => println!("Received PING frame {:?}", data),
///                 None => break println!("Pool closed"),
///             }
///         }
///     });
///
///     tokio::spawn(async move {
///         loop {
///             let data = pool3.read(HANDSHAKE_KIND).await;
///
///             match data {
///                 Some(data) => println!("Received HANDSHAKE frame {:?}", data),
///                 None => break println!("Pool closed"),
///             }
///         }
///     });
///
///     let ping_frame = Frame::create(PING_KIND, vec![]);
///     let handshake_frame = Frame::create(HANDSHAKE_KIND, vec![1, 2, 3]);
///
///     pool.write(ping_frame).await.unwrap();
///     time::sleep(time::Duration::from_secs(4)).await;
///
///     pool.write(handshake_frame).await.unwrap();
///     time::sleep(time::Duration::from_secs(4)).await;
///
///     pool.close().await;
/// }
/// ```
pub struct Pool<K: Eq + Hash, V: Kind<K>> {
    semaphore: Arc<Semaphore>,
    values: Arc<Mutex<ValueStore<K, V>>>,
}

impl<K: Eq + Hash, V: Kind<K>> Pool<K, V> {
    /// Creates new pool
    pub fn new() -> Self {
        Default::default()
    }

    /// Reads value with specific kind from pool
    ///
    /// Returns ['None'] if the pool was closed
    ///
    /// # Note
    ///
    /// Do not read one kind of data from several tasks,
    /// this will lead to the selection of a random receiver
    ///
    /// ['None']: std::option::Option::None
    pub async fn read(&self, kind: K) -> Option<V> {
        if let Some(value) = self.read_value(&kind).await {
            return Some(value);
        }

        loop {
            match self.semaphore.acquire().await {
                Ok(permit) => {
                    if let Some(value) = self.read_value(&kind).await {
                        permit.forget();
                        break Some(value);
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

    /// Reads any value written to the pool
    ///
    /// Returns ['None'] if the pool was closed
    ///
    /// # Note
    ///
    /// Do not mix ['read'] and ['read_any'] functions.
    /// This function adds a receiver for any value, so if you add another receiver
    /// for a specific type, a receiver recipient will be selected
    ///
    /// ['read']: crate::transport::pool::read
    /// ['read_any']: crate::transport::pool::read_any
    pub async fn read_any(&self) -> Option<V> {
        if let Some(value) = self.read_any_value().await {
            return Some(value);
        }

        match self.semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();

                // Always Some()
                self.read_any_value().await
            },

            // Pool is closed
            Err(_) => None,
        }
    }

    async fn read_any_value(&self) -> Option<V> {
        for (_, values) in self.values.lock().await.map.iter_mut() {
            if !values.is_empty() {
                // Always Some()
                return values.pop_front();
            }
        }
        None
    }

    /// Writes value to the pool
    ///
    /// Returns ['WriteError'] if the pool was closed
    ///
    /// ['WriteError']: crate::transport::pool::WriteError
    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
        {
            let mut value_store = self.values.lock().await;

            if !value_store.can_write {
                return Err(WriteError(value))
            }

            // Creating linked list if not exists and inserting frame
            value_store.map
                .entry(value.kind())
                .or_insert_with(VecDeque::new)
                .push_front(value);
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
            values: Arc::new(Mutex::new(ValueStore::new())),
        }
    }
}

struct ValueStore<K: Eq + Hash, V: Kind<K>> {
    map: HashMap<K, VecDeque<V>>,
    can_write: bool,
}

impl<K: Eq + Hash, V: Kind<K>> ValueStore<K, V> {
    fn new() -> Self {
        Default::default()
    }
}

impl<K: Eq + Hash, V: Kind<K>> Default for ValueStore<K, V> {
    fn default() -> Self {
        ValueStore {
            map: HashMap::new(),
            can_write: true,
        }
    }
}