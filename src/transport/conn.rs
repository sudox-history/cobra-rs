use tokio::net::TcpStream;
use std::io;
use std::hash::Hash;
use std::sync::Arc;

use crate::transport::pool::{PoolAny, Pool, Kind, WriteError};
use crate::transport::buffer::{Chunk, ConcatBuffer};

pub struct Conn<K, F>
where
    K: Eq + Hash + Send + 'static,
    F: Kind<K> + Chunk + Send + 'static
{
    write_pool: PoolAny<F>,
    read_pool: Pool<K, F>
}

impl<K, F> Conn<K, F>
    where
    K: Eq + Hash + Send + 'static,
    F: Kind<K> + Chunk + Send + 'static
{
    pub async fn connect(addr: &str) -> io::Result<Self> {
        let read_tcp_stream = Arc::new(TcpStream::connect(addr).await?);
        let write_tcp_stream = read_tcp_stream.clone();

        let self_read_pool = Pool::new();
        let read_pool: Pool<K, F>  = self_read_pool.clone();

        let self_write_pool = PoolAny::new();
        let write_pool: PoolAny<F> = self_write_pool.clone();

        let mut buffer: ConcatBuffer<F> = ConcatBuffer::default();

        // Read
        tokio::spawn(async move {
            loop {
                if read_tcp_stream.readable().await.is_err() {
                    break
                }

                match read_tcp_stream.try_read(&mut buffer) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                    Err(_) => break
                }

                if let Some(chunk) = buffer.try_read_chunk() {
                    if read_pool.write(chunk).await.is_err() {
                        break;
                    }
                }
            }
            read_pool.close().await;
        });

        // Write
        tokio::spawn(async move {
            while let Some(frame) = write_pool.read().await {
                match write_tcp_stream.writable().await {
                    Ok(_) => {}
                    Err(_) => break
                }

                match write_tcp_stream.try_write(&frame) {
                    Ok(_) => break,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                    Err(_) => break
                }
            }
            write_pool.close().await;
        });

        Ok(Conn {
            write_pool: self_write_pool,
            read_pool: self_read_pool,
        })
    }

    // Return None if connection close
    pub async fn read(&self, kind: K) -> Option<F> {
        self.read_pool.read(kind).await
    }

    // Return WriteError<F> if connection close
    pub async fn write(&self, frame: F) -> Result<(), WriteError<F>> {
        self.write_pool.write(frame).await
    }
}
