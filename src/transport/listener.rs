use crate::transport::conn::Conn;
use crate::transport::pool::PoolAny;
use tokio::net::{ToSocketAddrs, TcpListener};
use std::io;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct Listener {
    connections_pool: PoolAny<Conn>,
    tcp_listener: Arc<TcpListener>,
    close_notifier: Arc<Notify>,
}

impl Listener {
    pub async fn listen<T: ToSocketAddrs>(addr: T) -> io::Result<Self> {
        let tcp_listener = Arc::new(TcpListener::bind(addr).await?);
        let connections_pool = PoolAny::new();
        let close_notifier = Arc::new(Notify::new());

        tokio::spawn(Listener::accept_loop(
            tcp_listener.clone(),
            connections_pool.clone(),
            close_notifier.clone()
        ));

        Ok(Listener {
            tcp_listener,
            connections_pool,
            close_notifier
        })
    }

    async fn accept_loop(tcp_listener: Arc<TcpListener>,
                         connections_pool: PoolAny<Conn>,
                         close_notifier: Arc<Notify>) {
        while let Ok((socket, _)) = tcp_listener.accept().await {
            let conn = Conn::from_raw(socket,
                                          Some(close_notifier.clone())).await;
            if connections_pool.write(conn).await.is_err() {
                break
            }
        }
        connections_pool.close().await;
    }

    pub async fn accept(&self) -> Option<Conn> {
        self.connections_pool.read().await
    }

    async fn close_all_connections(&self) {
        self.close_notifier.notify_waiters();
    }
}
