use std::io;
use std::ops::DerefMut;
use std::sync::Arc;

use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Notify;

use crate::sync::{KindPool, Pool, WriteError};
use crate::transport::buffer::ConcatBuffer;
use crate::transport::frame::Frame;
use std::net::SocketAddr;

pub struct Conn {
    write_pool: Pool<Frame>,
    read_pool: KindPool<u8, Frame>,
    conn_close_notifier: Arc<Notify>,
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
}

impl Conn {
    pub(crate) async fn from_raw(tcp_stream: TcpStream,
                                 server_close_notifier: Option<Arc<Notify>>) -> io::Result<Self> {
        let local_addr = tcp_stream.local_addr()?;
        let peer_addr = tcp_stream.peer_addr()?;

        let read_tcp_stream = Arc::new(tcp_stream);
        let write_tcp_stream = read_tcp_stream.clone();

        let read_pool = KindPool::new();
        let write_pool = Pool::new();

        let buffer = ConcatBuffer::default();

        let conn_close_notifier = Arc::new(Notify::new());

        tokio::spawn(Conn::close_task(
            server_close_notifier,
            conn_close_notifier.clone(),
            read_pool.clone(),
            write_pool.clone(),
        ));

        tokio::spawn(Conn::read_loop(
            read_tcp_stream,
            read_pool.clone(),
            buffer,
        ));

        tokio::spawn(Conn::write_loop(
            write_tcp_stream,
            write_pool.clone(),
        ));

        Ok(Conn {
            write_pool,
            read_pool,
            conn_close_notifier,
            local_addr,
            peer_addr,
        })
    }

    pub async fn connect<T: ToSocketAddrs>(addr: T) -> io::Result<Self> {
        Conn::from_raw(TcpStream::connect(addr).await?, None).await
    }

    async fn close_task(server_close_notifier: Option<Arc<Notify>>,
                        conn_close_notifier: Arc::<Notify>,
                        read_pool: KindPool<u8, Frame>,
                        write_pool: Pool<Frame>) {
        match server_close_notifier {
            Some(server_close_notifier) => {
                tokio::select! {
                    _ = server_close_notifier.notified() => {}
                    _ = conn_close_notifier.notified() => {}
                }
            }
            None => {
                conn_close_notifier.notified().await;
            }
        }
        read_pool.close().await;
        write_pool.close();
    }

    async fn read_loop(read_tcp_stream: Arc<TcpStream>,
                       read_pool: KindPool<u8, Frame>,
                       mut buffer: ConcatBuffer<Frame>) {
        loop {
            if read_tcp_stream.readable().await.is_err() {
                break;
            }
            match read_tcp_stream.try_read_buf(buffer.deref_mut()) {
                Ok(0) => break,
                Ok(_) => {}
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(_) => break
            }

            while let Some(chunk) = buffer.try_read_chunk() {
                if read_pool.write(chunk).await.is_err() {
                    break;
                }
            }
        }
        read_pool.close().await;
    }

    async fn write_loop(write_tcp_stream: Arc<TcpStream>,
                        write_pool: Pool<Frame>) {
        while let Some(mut frame) = write_pool.read().await {
            while !frame.is_empty() {
                if write_tcp_stream.writable().await.is_err() {
                    break;
                }
                match write_tcp_stream.try_write(&frame) {
                    Ok(n) => {
                        **frame = frame.split_off(n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                    Err(_) => break
                }
            }
            if !frame.is_empty() {
                frame.reject().await;
            }
        }
        write_pool.close();
    }

    // Return None if connection close
    pub async fn read(&self, kind: u8) -> Option<Frame> {
        Some(self.read_pool
            .read(kind)
            .await?
            .accept())
    }

    // Return WriteError<F> if connection close
    pub async fn write(&self, frame: Frame) -> Result<(), WriteError<Frame>> {
        self.write_pool.write(frame).await
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    pub fn close(&self) {
        self.conn_close_notifier.notify_one();
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        self.conn_close_notifier.notify_one();
    }
}
