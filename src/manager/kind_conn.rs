use std::sync::Arc;

use crate::manager::context::ContextState;
use crate::sync::WriteError;

pub enum CloseCode {
    ClosedByUser,
    NotFoundPing,
    NotFoundEncryption,
    NotFoundCompression,
    PingTimeout,
    EncryptionError,
    CompressionError,
}

impl CloseCode {
    pub fn get_code(&self) -> u8 {
        match self {
            CloseCode::ClosedByUser => 0,
            CloseCode::NotFoundPing => 1,
            CloseCode::NotFoundEncryption => 2,
            CloseCode::NotFoundCompression => 3,
            CloseCode::PingTimeout => 4,
            CloseCode::EncryptionError => 5,
            CloseCode::CompressionError => 6,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code {
            0 => CloseCode::ClosedByUser,
            1 => CloseCode::NotFoundPing,
            2 => CloseCode::NotFoundEncryption,
            3 => CloseCode::NotFoundCompression,
            4 => CloseCode::PingTimeout,
            5 => CloseCode::EncryptionError,
            6 => CloseCode::CompressionError,
            _ => panic!("Code {} not found", code),
        }
    }
}

pub struct KindConn {
    kind: u8,
    context_state: Arc<ContextState>,
}

impl KindConn {
    pub(crate) fn new(kind: u8, context_state: Arc<ContextState>) -> Self {
        KindConn {
            kind,
            context_state,
        }
    }

    pub async fn read(&self) -> Option<Vec<u8>> {
        self.context_state.read(self.kind).await
    }

    pub async fn write(&self, package: Vec<u8>) -> Result<(), WriteError<Vec<u8>>> {
        self.context_state.write(self.kind, package).await
    }

    pub async fn close(&self, code: CloseCode) {
        self.context_state.close(code).await
    }

    pub async fn is_close(&self) -> Option<CloseCode> {
        self.context_state.is_close().await
    }
}
