use crate::transport::conn::Conn;
use crate::transport::sync::Pool;
use tokio::net::{ToSocketAddrs, TcpListener};
use std::io;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct Listener {
    connections_pool: Pool<Conn>,
    close_notifier: Arc<Notify>,
}

impl Listener {
    pub async fn listen<T: ToSocketAddrs>(addr: T) -> io::Result<Self> {
        let tcp_listener = Arc::new(TcpListener::bind(addr).await?);
        let connections_pool = Pool::new();
        let close_notifier = Arc::new(Notify::new());

        tokio::spawn(Listener::accept_loop(
            tcp_listener.clone(),
            connections_pool.clone(),
            close_notifier.clone()
        ));

        Ok(Listener {
            connections_pool,
            close_notifier
        })
    }

    async fn accept_loop(tcp_listener: Arc<TcpListener>,
                         connections_pool: Pool<Conn>,
                         close_notifier: Arc<Notify>) {
        while let Ok((socket, _)) = tcp_listener.accept().await {
            let conn = Conn::from_raw(socket,
                                          Some(close_notifier.clone())).await;
            if connections_pool.write(conn).await.is_err() {
                break
            }
        }
        connections_pool.close();
    }

    pub async fn accept(&self) -> Option<Conn> {
        Some(self.connections_pool
            .read()
            .await?
            .accept())
    }

    pub async fn close_all_connections(&self) {
        self.close_notifier.notify_waiters();
    }
}
