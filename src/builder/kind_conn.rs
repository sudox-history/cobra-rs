use std::net::SocketAddr;
use std::sync::Arc;

use crate::builder::context::{ContextMode, ContextState};
use crate::sync::WriteError;

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
    context_state: Arc<ContextState>,
}

impl KindConn {
    pub(crate) fn new(kind: u8, mode: ContextMode, context_state: Arc<ContextState>) -> Self {
        KindConn {
            kind,
            mode,
            context_state,
        }
    }

    pub async fn read(&self) -> Option<Vec<u8>> {
        self.context_state.read(self.kind).await
    }

    pub async fn write(&self, package: Vec<u8>) -> Result<(), WriteError<Vec<u8>>> {
        self.context_state.write(self.kind, package, self.mode).await
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.context_state.local_addr()
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.context_state.peer_addr()
    }

    pub async fn close(&self, code: u8) {
        self.context_state.close(code).await
    }

    pub async fn is_close(&self) -> Option<u8> {
        self.context_state.is_close().await
    }
}
