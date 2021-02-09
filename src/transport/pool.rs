use tokio::sync::{Semaphore, RwLock, Mutex};
use std::sync::Arc;
use tokio::sync::oneshot;
use oneshot::Sender;
use std::ops::{Deref, DerefMut};
use std::hash::Hash;
use std::collections::HashMap;

pub enum WriteError<V> {
    Denied(V),
    Closed(V),
}

struct PoolState<V> {
    read_semaphore: Semaphore,
    write_semaphore: Semaphore,
    value: RwLock<Option<(V, Sender<Result<(), V>>)>>,
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
                self.state.read_semaphore.add_permits(1);

                let (sender, receiver) = oneshot::channel();
                *self.state.value.write().await = Some((value, sender));

                if let Ok(res) = receiver.await {
                    match res {
                        Ok(_) => Ok(()),
                        Err(value) => Err(WriteError::Denied(value))
                    }
                }
                else {
                    panic!("Sender was dropped")
                }
            }

            Err(_) => Err(WriteError::Closed(value))
        }
    }

    pub async fn read(&self) -> Option<PoolOutput<V>> {
        match self.state.read_semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();
                self.state.write_semaphore.add_permits(1);

                match self.state.value.write().await.take() {
                    Some((value, sender)) => {
                        Some(PoolOutput::new(value, sender))
                    }
                    None => {
                        panic!("Read value is empty")
                    }
                }
            }

            Err(_) => None
        }
    }

    pub fn clone(&self) -> Self {
        Pool {
            state: self.state.clone()
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
    value: Option<V>,
    sender: Option<Sender<Result<(), V>>>,
}

impl<V> PoolOutput<V> {
    fn new(value: V, sender: Sender<Result<(), V>>) -> Self {
        PoolOutput {
            value: Some(value),
            sender: Some(sender),
        }
    }

    pub fn accept(&mut self) -> V {
        if self.value.is_none() {
            panic!("Double response error")
        }

        if self.sender
            .take()
            .unwrap()
            .send(Ok(())).is_err() {
            panic!("Receiver was dropped")
        }

        self.value.take().unwrap()
    }

    pub fn deny(&mut self) {
        if self.value.is_none() {
            panic!("Double response error")
        }

        if self.sender
            .take()
            .unwrap()
            .send(Err(self.value.take().unwrap()))
            .is_err() {
            panic!("Receiver was dropped")
        }
    }
}

impl<V> Drop for PoolOutput<V> {
    fn drop(&mut self) {
        if self.value.is_some() {
            panic!("PoolOutput was dropped without response")
        }
    }
}

impl<V> Deref for PoolOutput<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        match &self.value {
            Some(value) => value,
            None => {
                panic!("Value already taken from PoolOutput")
            }
        }
    }
}

impl<V> DerefMut for PoolOutput<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.value {
            Some(value) => value,
            None => {
                panic!("Value already taken from PoolOutput")
            }
        }
    }
}

pub trait Kind<T> {
    fn kind(&self) -> T;
}

pub struct KindPool<K: Eq + Hash, V: Kind<K>> {
    pools: Arc<Mutex<HashMap<K, Pool<V>>>>
}

impl<K: Eq + Hash, V: Kind<K>> KindPool<K, V> {
    pub fn new() -> Self {
        KindPool {
            pools: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn write(&self, value: V) -> Result<(), WriteError<V>> {
       self.pools
           .lock()
           .await
           .entry(value.kind())
           .or_insert_with(Pool::new)
           .write(value)
           .await
    }

    pub async fn read(&self, kind: K) -> Option<PoolOutput<V>> {
        self.pools
            .lock()
            .await
            .get_mut(&kind)?
            .read()
            .await
    }


    pub async fn close(&self) {
        for (_, pool) in self.pools.lock().await.iter() {
            pool.close();
        }
    }
}

impl<K: Eq + Hash, V: Kind<K>> Clone for KindPool<K, V> {
    fn clone(&self) -> Self {
        KindPool {
            pools: self.pools.clone()
        }
    }
}
