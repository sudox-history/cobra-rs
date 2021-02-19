use std::io;
use std::net::SocketAddr;

use async_trait::async_trait;
use tokio::net::ToSocketAddrs;
use tokio::sync::RwLock;

use crate::builder::builder::ConnProvider;
use crate::sync::WriteError;
use crate::transport::conn::Conn;
use crate::transport::frame::Frame;

pub struct TcpConnProvider {
    conn: Conn,
    error_code: RwLock<u8>,
}

impl TcpConnProvider {
    pub async fn new<T: ToSocketAddrs>(addr: T) -> io::Result<Self> {
        Ok(TcpConnProvider {
            conn: Conn::connect(&addr).await?,
            error_code: RwLock::new(0),
        })
    }
}

#[async_trait]
impl ConnProvider for TcpConnProvider {
    async fn read(&self, kind: u8) -> Option<Frame> {
        self.conn
            .read(kind)
            .await
    }

    async fn write(&self, frame: Frame) -> Result<(), WriteError<Frame>> {
        self.conn
            .write(frame)
            .await
    }

    fn local_addr(&self) -> SocketAddr {
        self.conn.local_addr()
    }

    fn peer_addr(&self) -> SocketAddr {
        self.conn.peer_addr()
    }

    async fn readable(&self) -> io::Result<()> {
        self.conn.readable().await
    }

    async fn close(&self, code: u8) {
        self.conn.close();
        *self.error_code.write().await = code;
    }

    async fn is_close(&self) -> Option<u8> {
        match *self.error_code.read().await {
            0 => None,
            n => Some(n),
        }
    }
}
