use std::sync::Arc;

use tokio::sync::RwLock;

use crate::manager::builder::{CompressionManager, ConnManager, EncryptionManager, PingManager};
use crate::sync::WriteError;
use crate::transport::frame::Frame;

struct ContextState {
    kind_counter: RwLock<u8>,
    conn: Arc<dyn ConnManager>,
    ping: Arc<dyn PingManager>,
    encryption: Arc<dyn EncryptionManager>,
    compression: Arc<dyn CompressionManager>,
}

impl ContextState {
    pub async fn read(&self, kind: u8) -> Option<Vec<u8>> {
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

    pub async fn write(&self, kind: u8, package: Vec<u8>) -> Result<(), WriteError<Vec<u8>>> {
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
}

pub struct Context {
    state: Arc<ContextState>,
}

impl Context {
    pub fn new(conn: Arc<dyn ConnManager>,
               ping: Arc<dyn PingManager>,
               encryption: Arc<dyn EncryptionManager>,
               compression: Arc<dyn CompressionManager>) -> Self {
        Context {
            state: Arc::new(ContextState {
                kind_counter: RwLock::new(1),
                conn,
                ping,
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

pub struct KindConn {
    kind: u8,
    context_state: Arc<ContextState>,
}

impl KindConn {
    fn new(kind: u8, context_state: Arc<ContextState>) -> Self {
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
}
