use std::{
    io,
    sync::Arc,
};

use tokio::{
    net::{ToSocketAddrs, TcpStream},
    sync::mpsc,
};

use crate::transport::buffer::FrameBuffer;


pub struct Connection {
    shared_state: Arc<SharedState>,
}

struct SharedState {
    tcp_stream: TcpStream,
    sender: tokio::sync::mpsc::Sender<FrameBuffer>,
}

impl Connection {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> io::Result<Connection> {
        let (sender, mut receiver) = mpsc::channel(100);
        let shared_state = Arc::new(
            SharedState {
                tcp_stream: TcpStream::connect(addr).await?,
                send
            }
        );


        // Cool code by bulat <3
        tokio::spawn(
            Connection::read_loop(shared_state.clone())
        );

        Ok(
            Connection {
                shared_state
            }
        )
    }

    fn close(&self) -> io::Result<()> {
        self.close()
    }
    
    async fn read_loop(shared_state: Arc<SharedState>) -> io::Result<()> {
        let mut buffer = FrameBuffer::new();
        loop {
            shared_state.tcp_stream.readable().await?;


            match shared_state.tcp_stream.try_read(buffer.get_slice_to_read()) {
                Ok(n) => {
                    buffer.readed_bytes(n);
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }

            while let Ok(new_frame) = buffer.try_get_frame() {
                 shared_state.sender.send(new_frame);
            }

            todo!();
            // TODO: Реализуем жопу
        }
    }
}
