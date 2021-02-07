use tokio::net::TcpStream;
use std::io;
use std::sync::Arc;

use crate::transport::pool::{PoolAny, Pool, WriteError};
use crate::transport::buffer::ConcatBuffer;
use crate::transport::frame::Frame;

pub struct Conn {
    write_pool: PoolAny<Frame>,
    read_pool: Pool<u8, Frame>
}

impl Conn {
    pub async fn connect(addr: &str) -> io::Result<Self> {
        let read_tcp_stream = Arc::new(TcpStream::connect(addr).await?);
        let write_tcp_stream = read_tcp_stream.clone();

        let self_read_pool = Pool::new();
        let read_pool  = self_read_pool.clone();

        let self_write_pool : PoolAny<Frame> = PoolAny::new();
        let write_pool = self_write_pool.clone();

        let buffer = ConcatBuffer::default();

        tokio::spawn(Conn::read_loop(read_tcp_stream, read_pool, buffer));

        tokio::spawn(Conn::write_loop(write_tcp_stream, write_pool));

        Ok(Conn {
            write_pool: self_write_pool,
            read_pool: self_read_pool,
        })
    }

    async fn read_loop(read_tcp_stream: Arc<TcpStream>,
                       read_pool: Pool<u8, Frame>,
                       mut buffer: ConcatBuffer<Frame>) {
        loop {
            if read_tcp_stream.readable().await.is_err() {
                break;
            }

            match read_tcp_stream.try_read(&mut buffer) {
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
                        write_pool: PoolAny<Frame>) {
        while let Some(frame) = write_pool.read().await {
            if write_tcp_stream.writable().await.is_err() {
                break;
            }

            match write_tcp_stream.try_write(&frame) {
                Ok(_) => break,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(_) => break
            }
        }
        write_pool.close().await;
    }

    // Return None if connection close
    pub async fn read(&self, kind: u8) -> Option<Frame> {
        self.read_pool.read(kind).await
    }

    // Return WriteError<F> if connection close
    pub async fn write(&self, frame: Frame) -> Result<(), WriteError<Frame>> {
        self.write_pool.write(frame).await
    }

    pub async fn close(&self) {
        self.write_pool.close().await;
        self.read_pool.close().await;
    }
}
