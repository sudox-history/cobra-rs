use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use tokio::sync::{RwLock, Semaphore};

pub enum WriteError<V> {
    Denied(V),
    Closed(V),
}

pub struct Pool<V> {
    state: Arc<PoolState<V>>
}

impl<V> Pool<V> {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
        match self.state.write_semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();

                *self.state.value.write().await = Some(value);
                self.state.read_semaphore.add_permits(1);

                let x = self.state.accept_semaphore.acquire().await;
                if let Ok(permit) = x {
                    permit.forget();
                } else {
                    return Err(WriteError::Closed(self.state
                        .value
                        .write()
                        .await
                        .take()
                        .unwrap()));
                }

                match self.state.value.write().await.take() {
                    None => {
                        self.state.write_semaphore.add_permits(1);
                        Ok(())
                    }
                    Some(value) => {
                        self.state.write_semaphore.add_permits(1);
                        Err(WriteError::Denied(value))
                    }
                }
            }
            Err(_) => Err(WriteError::Closed(value))
        }
    }

    pub async fn read(&self) -> Option<PoolOutput<V>> {
        self.state.read_semaphore
            .acquire()
            .await
            .ok()?
            .forget();
        let value = self.state.value.write().await.take().unwrap();
        Some(PoolOutput::new(value, self.state.clone()))
    }

    pub fn close(&self) {
        self.state.read_semaphore.close();
        self.state.write_semaphore.close();
        self.state.accept_semaphore.close();
    }
}

impl<V> Clone for Pool<V> {
    fn clone(&self) -> Self {
        Pool {
            state: self.state.clone()
        }
    }
}

impl<V> Default for Pool<V> {
    fn default() -> Self {
        Pool {
            state: Arc::new(PoolState::new())
        }
    }
}

struct PoolState<V> {
    write_semaphore: Semaphore,
    read_semaphore: Semaphore,
    accept_semaphore: Semaphore,
    value: RwLock<Option<V>>,
}

impl<V> PoolState<V> {
    fn new() -> Self {
        Default::default()
    }
}

impl<V> Default for PoolState<V> {
    fn default() -> Self {
        PoolState {
            write_semaphore: Semaphore::new(1),
            read_semaphore: Semaphore::new(0),
            accept_semaphore: Semaphore::new(0),
            value: RwLock::new(None),
        }
    }
}

pub struct PoolOutput<V> {
    value: V,
    state: Arc<PoolState<V>>,
}

impl<V> PoolOutput<V> {
    fn new(value: V, state: Arc<PoolState<V>>) -> Self {
        PoolOutput {
            value,
            state,
        }
    }

    pub fn accept(self) -> V {
        self.state.accept_semaphore.add_permits(1);
        self.value
    }

    pub async fn deny(self) {
        *self.state.value.write().await = Some(self.value);
        self.state.accept_semaphore.add_permits(1);
    }
}

impl<V> Deref for PoolOutput<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V> DerefMut for PoolOutput<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
