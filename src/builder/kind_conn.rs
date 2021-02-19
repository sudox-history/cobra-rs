use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::builder::context::{ContextMode, ContextState};
use crate::sync::WriteError;
use crate::transport::frame::Frame;

pub mod close_code {
    pub const CLOSED_BY_USER: u8 = 1;
    pub const NOT_FOUND_PING: u8 = 2;
    pub const NOT_FOUND_ENCRYPTION: u8 = 3;
    pub const NOT_FOUND_COMPRESSION: u8 = 4;
    pub const PING_TIMEOUT: u8 = 5;
    pub const ENCRYPTION_ERROR: u8 = 6;
    pub const COMPRESSION_ERROR: u8 = 7;
}

pub struct KindConn {
    kind: u8,
    mode: ContextMode,
    state: Arc<ContextState>,
}

impl KindConn {
    pub(crate) fn new(kind: u8, mode: ContextMode, state: Arc<ContextState>) -> Self {
        KindConn {
            kind,
            mode,
            state,
        }
    }

    pub async fn read(&self) -> Option<Vec<u8>> {
        let package = self.state
            .conn
            .read(self.kind)
            .await?
            .get_data();
        let package = self.state
            .compression
            .decompress(package);
        let package = self.state
            .encryption
            .decrypt(package);

        Some(package)
    }

    pub async fn write(&self, package: Vec<u8>) -> Result<(), WriteError<Vec<u8>>> {
        let frame = match self.mode {
            ContextMode::Raw => Frame::new(self.kind, package),
            ContextMode::Handle => {
                let package = self.state
                    .encryption
                    .encrypt(package);
                let package = self.state
                    .compression
                    .compress(package);
                Frame::new(self.kind, package)
            }
        };

        self.state
            .conn
            .write(frame)
            .await
            .map_err(|err| err.map(|frame| frame.get_data()))
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.state.conn.local_addr()
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.state.conn.peer_addr()
    }

    pub async fn readable(&self) -> io::Result<()> {
        self.state.conn.readable().await
    }

    pub async fn close(&self, code: u8) {
        self.state.conn.close(code).await
    }

    pub async fn is_close(&self) -> Option<u8> {
        self.state.conn.is_close().await
    }
}
