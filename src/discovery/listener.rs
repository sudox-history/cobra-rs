use std::net::Ipv4Addr;
use std::sync::Arc;

use tokio::sync::Notify;

use crate::discovery::default_values::{DEFAULT_ADDRESS, DEFAULT_MULTICAST_ADDRESS, DEFAULT_PORT};
use crate::discovery::default_values::{DEFAULT_ANSWER_PACKAGE, DEFAULT_SEARCH_PACKAGE};
use crate::discovery::search_socket::SearchSocket;

pub struct Listener {
    close_notifier: Option<Arc<Notify>>,
    socket: Arc<SearchSocket>,
}

impl Listener {
    pub async fn new() -> std::io::Result<Self> {
        Self::custom(DEFAULT_ADDRESS, DEFAULT_MULTICAST_ADDRESS, DEFAULT_PORT).await
    }

    pub async fn custom(addr: Ipv4Addr, multi_addr: Ipv4Addr, port: u16) -> std::io::Result<Self> {
        let socket = Arc::new(SearchSocket::new(addr, multi_addr, port).await?);
        let close_notifier = Self::spawn(socket.clone());
        Ok(Listener {
            close_notifier: Some(close_notifier),
            socket,
        })
    }

    pub fn is_active(&self) -> bool {
        self.close_notifier.is_some()
    }

    pub fn pause(&mut self) {
        if let Some(close_notifier) = self.close_notifier.take() {
            close_notifier.notify_one();
        }
    }

    pub fn resume(&mut self) {
        if self.close_notifier.is_none() {
            self.close_notifier = Some(Self::spawn(self.socket.clone()));
        }
    }

    fn spawn(socket: Arc<SearchSocket>) -> Arc<Notify> {
        let close_notifier = Arc::new(Notify::new());
        let out_close_notifier = close_notifier.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = Self::receive_and_answer(&socket) => {}
                    _ = close_notifier.notified() => { break }
                }
            }
        });
        out_close_notifier
    }

    async fn receive_and_answer(socket: &SearchSocket) {
        if let Ok((data, _)) = socket.read().await {
            if data == DEFAULT_SEARCH_PACKAGE {
                socket.send(DEFAULT_ANSWER_PACKAGE.to_vec()).await.unwrap();
            }
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(close_notifier) = &self.close_notifier {
            close_notifier.notify_one();
        }
    }
}
