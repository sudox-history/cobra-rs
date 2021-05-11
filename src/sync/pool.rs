use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::{Notify, RwLock, Semaphore};

/// Error returned on [`write`] failure
///
/// [`write`]: crate::transport::sync::Pool::write
#[derive(Debug)]
pub enum WriteError<T> {
    /// Value reject by user
    Rejected(T),

    /// Pool is closed
    Closed(T),
}

impl<T> WriteError<T> {
    pub fn map<F, O: FnOnce(T) -> F>(self, op: O) -> WriteError<F> {
        match self {
            WriteError::Rejected(e) => WriteError::Rejected(op(e)),
            WriteError::Closed(e) => WriteError::Rejected(op(e)),
        }
    }
}

/// Asynchronous value pool
///
/// Can be used to atomically transfer data between tasks
///
/// # Example
///
/// ```
/// use cobra_rs::sync::{Pool};
///
/// #[tokio::main]
/// async fn main() {
///     let pool = Pool::new();
///     let pool2: Pool<i32> = pool.clone();
///
///     tokio::spawn(async move {
///         loop {
///             match pool2.read().await {
///                 Some(value) => {
///                     println!("Received value: {:?}", *value);
///                     value.accept();
///                 }
///                 None => {
///                     println!("Pool closed");
///                     break;
///                 }
///             }
///         }
///     });
///
///     pool.write(12).await.unwrap();
///     pool.close();
/// }
/// ```
pub struct Pool<T> {
    state: Arc<PoolState<T>>,
}

struct PoolState<T> {
    read_semaphore: Semaphore,
    write_semaphore: Semaphore,
    response_notifier: Notify,
    close_notifier: Notify,
    store: RwLock<Option<T>>,
}

/// Value returned by [`read`] method
///
/// [`read`]: crate::transport::sync::Pool::read
pub struct PoolGuard<T> {
    value: Option<T>,
    state: Arc<PoolState<T>>,
}

impl<T> Pool<T> {
    /// Creates a new pool
    pub fn new() -> Self {
        Default::default()
    }

    /// Reads value from the pool
    ///
    /// Returns [`PoolGuard`], which can be used to accept or reject
    /// the value and [`None`] if the pool was closed
    ///
    /// [`None`]: std::option::Option::None
    /// [`PoolGuard`]: crate::transport::pool::PoolGuard
    pub async fn read(&self) -> Option<PoolGuard<T>> {
        Some(PoolGuard::new(
            self.state.read_value().await.ok()?,
            self.state.clone(),
        ))
    }

    /// Writes value to the pool
    ///
    /// Unlocks only when reader has been accepted or rejected.
    /// Returns [`WriteError`] if the value was rejected by another side or
    /// the pool was closed
    ///
    /// [`WriteError`]: crate::transport::pool::WriteError
    pub async fn write(&self, value: T) -> Result<(), WriteError<T>> {
        self.state
            .write_value(value)
            .await
            .map_err(WriteError::Closed)?;

        self.state
            .wait_response()
            .await
            .map_err(WriteError::Closed)?
            .map_or(Ok(()), |value| Err(WriteError::Rejected(value)))
    }

    /// Closes the pool
    pub fn close(&self) {
        self.state.close();
    }
}

impl<T> PoolState<T> {
    fn new() -> Self {
        PoolState {
            read_semaphore: Semaphore::new(0),
            write_semaphore: Semaphore::new(1),
            response_notifier: Notify::new(),
            close_notifier: Notify::new(),
            store: RwLock::new(None),
        }
    }

    async fn read_value(&self) -> Result<T, ()> {
        self.read_semaphore.acquire().await.or(Err(()))?.forget();

        // Always Some()
        Ok(self.take().await.unwrap())
    }

    async fn write_value(&self, value: T) -> Result<(), T> {
        match self.write_semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();

                self.share(value).await;
                self.read_semaphore.add_permits(1);

                Ok(())
            }

            Err(_) => Err(value),
        }
    }

    async fn wait_response(&self) -> Result<Option<T>, T> {
        let closed = tokio::select! {
            _ = self.response_notifier.notified() => { false }
            _ = self.close_notifier.notified() => { true }
        };

        if closed {
            Err(self.take().await.unwrap())
        } else {
            let value = self.take().await;
            self.write_semaphore.add_permits(1);
            Ok(value)
        }
    }

    async fn share(&self, value: T) {
        *self.store.write().await = Some(value);
    }

    async fn take(&self) -> Option<T> {
        let mut store = self.store.write().await;
        store.take()
    }

    fn close(&self) {
        if let Ok(permit) =  self.read_semaphore.try_acquire() {
            permit.forget();
            self.close_notifier.notify_one();
        }
        self.read_semaphore.close();
        self.write_semaphore.close();
    }
}

impl<T> PoolGuard<T> {
    fn new(value: T, state: Arc<PoolState<T>>) -> Self {
        PoolGuard {
            value: Some(value),
            state,
        }
    }

    /// Accepts value from the pool
    ///
    /// This will cause writer to unlock with [`Ok`] result
    ///
    /// # Note
    ///
    /// If [`PoolGuard`] has dropped, it will automatically accept the value
    ///
    /// [`Ok`]: std::result::Result::Ok
    /// [`PoolGuard`]: crate::transport::pool::PoolGuard
    pub fn accept(mut self) -> T {
        self.state.response_notifier.notify_one();

        // Always Some()
        self.value.take().unwrap()
    }

    /// Rejects value from the pool
    ///
    /// This will cause writer to unlock with [`WriteError::Rejected`] result
    ///
    /// [`WriteError::Rejected`]: crate::transport::sync::WriteError
    pub async fn reject(mut self) {
        self.state.share(self.value.take().unwrap()).await;

        self.state.response_notifier.notify_one();
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Pool {
            state: Arc::new(PoolState::new()),
        }
    }
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Pool {
            state: self.state.clone(),
        }
    }
}

impl<T> Deref for PoolGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T> Drop for PoolGuard<T> {
    fn drop(&mut self) {
        if self.value.take().is_some() {
            self.state.response_notifier.notify_one();
        }
    }
}
