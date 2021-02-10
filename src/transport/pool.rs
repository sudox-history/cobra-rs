use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use oneshot::Sender;
use tokio::sync::{RwLock, Semaphore};
use tokio::sync::oneshot;

pub enum WriteError<V> {
    Denied(V),
    Closed(V),
}

struct PoolState<V> {
    read_semaphore: Semaphore,
    write_semaphore: Semaphore,
    value: RwLock<Option<V>>,
}

impl<V> PoolState<V> {
    fn new() -> Self {
        PoolState {
            read_semaphore: Semaphore::new(0),
            write_semaphore: Semaphore::new(1),
            value: RwLock::new(None),
        }
    }
}

pub struct Pool<V> {
    state: Arc<PoolState<V>>,
}

impl<V> Pool<V> {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
        match self.state.write_semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();

                let (sender, receiver) = oneshot::channel();
                *self.state.value.write().await = Some((value, sender));

                self.state.read_semaphore.add_permits(1);

                if let Ok(res) = receiver.await {
                    match res {
                        Ok(_) => Ok(()),
                        Err(value) => Err(WriteError::Denied(value))
                    }
                }
                // Frame was dropped without accept/deny
                else {
                    Ok(())
                }
            }

            Err(_) => Err(WriteError::Closed(value))
        }
    }

    pub async fn read(&self) -> Option<PoolOutput<V>> {
        match self.state.read_semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();

                let out = match self.state.value.write().await.take() {
                    Some((value, sender)) => {
                        Some(PoolOutput::new(value, sender))
                    }
                    None => {
                        panic!("Read value is empty")
                    }
                };
                self.state.write_semaphore.add_permits(1);
                out
            }

            Err(_) => None
        }
    }

    pub fn close(&self) {
        self.state.read_semaphore.close();
        self.state.write_semaphore.close();
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

pub struct PoolOutput<V> {
    value: V,
    sender: Sender<Result<(), V>>,
}

impl<V> PoolOutput<V> {
    fn new(value: V, sender: Sender<Result<(), V>>) -> Self {
        PoolOutput {
            value,
            sender,
        }
    }

    pub fn accept(self) -> V {
        if self.sender.send(Ok(())).is_err() {
            panic!("Receiver was dropped")
        }

        self.value
    }

    pub fn deny(self) {
        if self.sender
            .send(Err(self.value))
            .is_err() {
            panic!("Receiver was dropped")
        }
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
