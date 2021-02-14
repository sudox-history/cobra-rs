use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use tokio::sync::{RwLock, Semaphore};

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
    state: Arc<PoolState<T>>
}

struct PoolState<T> {
    read_semaphore: Semaphore,
    write_semaphore: Semaphore,
    response_semaphore: Semaphore,
    value: RwLock<Option<T>>,
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

    /// Writes value to the pool
    ///
    /// Unlocks only when reader has been accepted or rejected.
    /// Returns [`WriteError`] if the value was rejected by another side or
    /// the pool was closed
    ///
    /// [`WriteError`]: crate::transport::pool::WriteError
    pub async fn write(&self, value: T) -> Result<(), WriteError<T>> {
        match self.state.write_value(value).await {
            None => match self.state.wait_response().await {
                Some(value) => Err(WriteError::Rejected(value)),
                None => Ok(())
            },

            // Pool closed
            Some(value) => Err(WriteError::Closed(value)),
        }
    }

    /// Reads value from the pool
    ///
    /// Returns [`PoolGuard`], which can be used to accept or reject
    /// the value and [`None`] if the pool was closed
    ///
    /// [`None`]: std::option::Option::None
    /// [`PoolGuard`]: crate::transport::pool::PoolGuard
    pub async fn read(&self) -> Option<PoolGuard<T>> {
        Some(
            PoolGuard::new(
                self.state.read_value().await?,
                self.state.clone(),
            )
        )
    }

    /// Closes the pool
    pub fn close(&self) {
        self.state.close()
    }
}

impl<T> PoolState<T> {
    fn new() -> Self {
        PoolState {
            read_semaphore: Semaphore::new(0),
            write_semaphore: Semaphore::new(1),
            response_semaphore: Semaphore::new(0),
            value: RwLock::new(None),
        }
    }

    async fn write_value(&self, value: T) -> Option<T> {
        match self.write_semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();

                *self.value.write().await = Some(value);
                self.read_semaphore.add_permits(1);

                None
            }
            _ => Some(value),
        }
    }

    async fn read_value(&self) -> Option<T> {
        self.read_semaphore.acquire().await.ok()?.forget();
        self.value.write().await.take()
    }

    fn accept_value(&self) {
        self.response_semaphore.add_permits(1);
    }

    async fn reject_value(&self, value: T) {
        *self.value.write().await = Some(value);
        self.response_semaphore.add_permits(1);
    }

    async fn wait_response(&self) -> Option<T> {
        self.response_semaphore.acquire().await.ok()?.forget();

        let value = self.value.write().await.take();
        self.write_semaphore.add_permits(1);

        value
    }

    fn close(&self) {
        self.read_semaphore.close();
        self.write_semaphore.close();
        self.response_semaphore.close();
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
        self.state.accept_value();

        // Always Some()
        self.value.take().unwrap()
    }

    /// Rejects value from the pool
    ///
    /// This will cause writer to unlock with [`WriteError::Rejected`] result
    ///
    /// [`WriteError::Rejected`]: crate::transport::sync::WriteError
    pub async fn reject(mut self) {
        self.state.reject_value(self.value.take().unwrap()).await;
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Pool {
            state: Arc::new(PoolState::new())
        }
    }
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Pool {
            state: self.state.clone()
        }
    }
}

impl<T> Deref for PoolGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T> DerefMut for PoolGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

impl<T> Drop for PoolGuard<T> {
    fn drop(&mut self) {
        if self.value.take().is_some() {
            self.state.accept_value();
        }
    }
}
