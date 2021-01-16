use std::{io, net::Shutdown, sync::Arc};

use tokio::{
    net::{TcpStream, ToSocketAddrs},
    sync::{broadcast, oneshot},
};

use crate::transport::buffer::FrameBuffer;
use crate::transport::frame::Frame;

use super::frame;

pub struct Connection {
    shared_state: Arc<SharedState>,
    receiver: broadcast::Receiver<Frame>,
    close_sender: oneshot::Sender<bool>, 
}

struct SharedState {
    tcp_stream: TcpStream,
    sender: broadcast::Sender<Frame>,
    close_receiver: oneshot::Receiver<bool>
}

impl Connection {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> io::Result<Connection> {
        let (sender, mut receiver) = broadcast::channel(100);
        let (close_sender, mut close_receiver) = oneshot::channel();
        let shared_state = Arc::new(SharedState {
            tcp_stream: TcpStream::connect(addr).await?,
            sender,
            close_receiver,
        });

        // Cool code by bulat <3
        tokio::spawn(Connection::read_loop(shared_state.clone()));

        Ok(Connection {
            shared_state,
            receiver,
            close_sender,
        })
    }

    async fn read(&mut self) -> io::Result<Frame> {
        match self.receiver.recv().await {
            Ok(frame) => frame,
            Err(_) => Err(io::Error::from(io::ErrorKind::NotConnected)),
        }
    }

    fn write(&self, frame: Frame) {}

    fn drop(self) {
        self.close_sender.send(true).unwrap();
    }
    
    async fn read_loop(shared_state: Arc<SharedState>) -> io::Result<()> {
        let mut buffer = FrameBuffer::new(100000, 2);
        loop {
            tokio::select!{
                _ = shared_state.tcp_stream.readable() => {

                }
                _ = shared_state.close_receiver.try_recv() => {

                }
            };

            // TODO: Handle queue overflow
            // TODO: Update ping timer

            match shared_state.tcp_stream.try_read(buffer.get_slice_to_read()) {
                Ok(n) => {
                    buffer.readed_bytes(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }

            while let Some(new_frame) = buffer.try_get_frame() {
                shared_state.sender.send(new_frame).unwrap();
            }
        }
    }
}
