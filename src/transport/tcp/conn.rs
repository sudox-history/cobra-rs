use std::io;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Notify;
use tokio::time;

use crate::mem::{ConcatBuf, Frame};
use crate::sync::{KindPool, Pool, WriteError};

pub struct Conn {
    inner: Arc<TcpStream>,

    // I/O loops
    reader: ConnReader,
    writer: ConnWriter,
}

struct ConnReader {
    pool: KindPool<u8, Frame>,
}

struct ConnWriter {
    pool: Pool<Frame>,
}

impl Conn {
    /// Tries to connect to the specified address
    ///
    /// # Note
    ///
    /// This function doesn't have any timeout.
    /// If you need to set timeout see [`connect_timeout()`]
    ///
    /// [`connect_timeout()`]: crate::transport::tcp::Conn::connect_timeout
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> io::Result<Self> {
        Ok(Conn::from_raw(TcpStream::connect(addr).await?))
    }

    /// Tries to connect to the specified address
    /// The same as [`connect()`] but requires a timeout
    ///
    /// [`connect()`]: crate::transport::tcp::Conn::connect()
    pub async fn connect_timeout<T: ToSocketAddrs>(addr: T, timeout: Duration) -> io::Result<Self> {
        Ok(
            Conn::from_raw(
                time::timeout(timeout, TcpStream::connect(addr)).await
                    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "connection timed out"))??
            )
        )
    }

    /// Returns local address that connection bound to
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    /// Returns remote address that connection connected to
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.inner.peer_addr()
    }

    /// Reads a frame from a connection
    ///
    /// Returns [`Frame`] on a successful read and [`None`] if
    /// connection was closed
    ///
    /// # Note
    ///
    /// This function is thread-safe and can be called from
    /// multiple tasks
    ///
    /// [`Frame`]: crate::mem::Frame
    /// [`None`]: std::option::Option::None
    pub async fn read(&self, kind: u8) -> Option<Frame> {
        self.reader.read(kind).await
    }

    /// Writes a frame to the connection
    ///
    /// Returns [`WriteError::Rejected`] if the packet wasn't written correctly
    /// (occurs only if a write attempt was made when the connection was closing)
    /// and [`WriteError::Closed`] if the connection was already closed
    ///
    /// # Note
    ///
    /// This function is thread-safe and can be called from
    /// multiple tasks
    ///
    /// [`WriteError::Rejected`]: crate::sync::WriteError::Rejected
    /// [`WriteError::Closed`]: crate::sync::WriteError::Closed
    pub async fn write(&self, frame: Frame) -> Result<(), WriteError<Frame>> {
        self.writer.write(frame).await
    }

    pub async fn close(self) {}

    pub(crate) fn from_raw(tcp_stream: TcpStream) -> Self {
        let inner = Arc::new(tcp_stream);

        Conn {
            inner: inner.clone(),
            reader: ConnReader::create(inner.clone()),
            writer: ConnWriter::create(inner),
        }
    }
}

impl ConnReader {
    fn create(inner: Arc<TcpStream>) -> Self {
        let worker = ConnReader {
            pool: KindPool::new(),
        };

        worker.spawn(inner);
        worker
    }

    fn spawn(&self, inner: Arc<TcpStream>) {
        let pool = self.pool.clone();

        tokio::spawn(async move {
            let mut buf = ConcatBuf::default();

            loop {
                if inner.readable().await.is_err() {
                    break;
                }

                match inner.try_read_buf(buf.deref_mut()) {
                    // On EOF closing read worker
                    Ok(0) => break,

                    // Ok
                    Ok(_len) => {}

                    // Operation can't be completed now and we should retry it
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,

                    // Closing read worker on unexpected error
                    Err(_) => break,
                }

                while let Some(frame) = buf.try_read_chunk() {
                    if pool.write(frame).await.is_err() {
                        break;
                    }
                }
            }

            pool.close().await;
        });
    }

    async fn read(&self, kind: u8) -> Option<Frame> {
        Some(self.pool.read(kind).await?.accept())
    }

    async fn close(&self) {
        self.pool.close().await
    }
}

impl ConnWriter {
    fn create(inner: Arc<TcpStream>) -> Self {
        let worker = ConnWriter {
            pool: Pool::new(),
        };

        worker.spawn(inner);
        worker
    }

    fn spawn(&self, inner: Arc<TcpStream>) {
        let pool = self.pool.clone();

        tokio::spawn(async move {
            while let Some(frame) = pool.read().await {
                let mut wrote_len = 0;

                while wrote_len < frame.len() {
                    if inner.writable().await.is_err() {
                        frame.reject().await;
                        break;
                    }

                    match inner.try_write(&frame[wrote_len..]) {
                        // Ok
                        Ok(len) => wrote_len += len,

                        // Operation can't be completed now and we should retry it
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,

                        // Closing write worker on unexpected error
                        Err(_) => {
                            frame.reject().await;
                            break;
                        }
                    }
                }
            }

            pool.close().await;
        });
    }

    async fn write(&self, frame: Frame) -> Result<(), WriteError<Frame>> {
        self.pool.write(frame).await
    }

    async fn close(&self) {
        self.pool.close().await
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        // Close connection
    }
}
