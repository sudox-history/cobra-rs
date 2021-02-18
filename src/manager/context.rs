use std::sync::Arc;

use tokio::sync::RwLock;

use crate::manager::builder::{CompressionManager, ConnManager, EncryptionManager};
use crate::sync::WriteError;
use crate::transport::frame::Frame;
use crate::manager::kind_conn::{KindConn, CloseCode};

pub(crate) struct ContextState {
    kind_counter: RwLock<u8>,
    conn: Arc<dyn ConnManager>,
    encryption: Arc<dyn EncryptionManager>,
    compression: Arc<dyn CompressionManager>,
}

impl ContextState {
    pub(crate) async fn read(&self, kind: u8) -> Option<Vec<u8>> {
        let package = self.conn
            .read(kind)
            .await?
            .get_data();
        let package = self.compression
            .decompress(package);
        let package = self.encryption
            .decrypt(package);

        Some(package)
    }

    pub(crate) async fn write(&self, kind: u8, package: Vec<u8>) -> Result<(), WriteError<Vec<u8>>> {
        let package = self.encryption
            .encrypt(package);
        let package = self.compression
            .compress(package);
        let frame = Frame::new(kind, package);

        self.conn
            .write(frame)
            .await
            .map_err(|err| err.map(|frame| frame.get_data()))
    }

    pub(crate) async fn close(&self, code: CloseCode) {
        self.conn.close(code).await
    }

    pub(crate) async fn is_close(&self) -> Option<CloseCode> {
        self.conn.is_close().await
    }
}

pub struct Context {
    state: Arc<ContextState>,
}

impl Context {
    pub fn new(conn: Arc<dyn ConnManager>,
               encryption: Arc<dyn EncryptionManager>,
               compression: Arc<dyn CompressionManager>) -> Self {
        Context {
            state: Arc::new(ContextState {
                kind_counter: RwLock::new(1),
                conn,
                encryption,
                compression,
            })
        }
    }

    pub async fn get_kind_conn(&self) -> KindConn {
        *self.state.kind_counter.write().await += 1;
        let kind = *self.state.kind_counter.read().await - 1;
        KindConn::new(kind, self.state.clone())
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Context {
            state: self.state.clone(),
        }
    }
}
