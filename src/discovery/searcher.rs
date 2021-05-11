use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;

use crate::discovery::default_values::{DEFAULT_ADDRESS, DEFAULT_MULTICAST_ADDRESS, DEFAULT_PORT};
use crate::discovery::default_values::{DEFAULT_ANSWER_PACKAGE, DEFAULT_SEARCH_PACKAGE};
use crate::discovery::search_socket::SearchSocket;
use crate::sync::Pool;

struct Searcher {
    pool: Pool<SocketAddr>,
    close_notifier: Arc<Notify>,
}

impl Searcher {
    pub async fn new(search_ratio: Duration) -> std::io::Result<Self> {
        Self::custom(
            DEFAULT_ADDRESS,
            DEFAULT_MULTICAST_ADDRESS,
            DEFAULT_PORT,
            search_ratio,
        )
        .await
    }

    pub async fn custom(
        addr: Ipv4Addr,
        multi_addr: Ipv4Addr,
        port: u16,
        search_ratio: Duration,
    ) -> std::io::Result<Self> {
        let socket = Arc::new(SearchSocket::new(addr, multi_addr, port).await?);
        let (pool, close_notifier) = Self::spawn(socket, search_ratio);

        Ok(Searcher {
            pool,
            close_notifier,
        })
    }

    pub async fn scan(&self) -> SocketAddr {
        self.pool
            .read()
            .await
            .unwrap()
            .accept()
    }

    fn spawn(socket: Arc<SearchSocket>, search_ratio: Duration) -> (Pool<SocketAddr>, Arc<Notify>) {
        let pool = Pool::new();
        let close_notifier = Arc::new(Notify::new());
        let mutex = Arc::new(Mutex::new(()));

        tokio::spawn(Self::sender_loop(
            socket.clone(),
            search_ratio,
            close_notifier.clone(),
            mutex.clone(),
        ));
        tokio::spawn(Self::receiver_loop(socket, pool.clone(), mutex));

        (pool, close_notifier)
    }

    async fn sender_loop(
        socket: Arc<SearchSocket>,
        search_ratio: Duration,
        close_notifier: Arc<Notify>,
        mutex: Arc<Mutex<()>>,
    ) {
        loop {
            drop(mutex.lock().await);
            tokio::select! {
                _ = close_notifier.notified() => {}
                _ = socket.send(DEFAULT_SEARCH_PACKAGE.to_vec()) => {}
            }
            sleep(search_ratio).await;
        }
    }

    async fn receiver_loop(
        socket: Arc<SearchSocket>,
        pool: Pool<SocketAddr>,
        mutex: Arc<Mutex<()>>,
    ) {
        loop {
            if let Ok((data, addr)) = socket.read().await {
                if data == DEFAULT_ANSWER_PACKAGE {
                    let lock = mutex.lock().await;
                    if pool.write(addr).await.is_err() {
                        break;
                    }
                    drop(lock);
                }
            }
        }
    }
}

impl Drop for Searcher {
    fn drop(&mut self) {
        self.close_notifier.notify_one();
        self.pool.close();
    }
}
