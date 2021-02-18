use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::builder::builder::{CompressionProvider, ConnProvider, EncryptionProvider};
use crate::builder::kind_conn::KindConn;
use crate::sync::WriteError;
use crate::transport::frame::Frame;

pub struct Context {
    state: Arc<ContextState>,
    mode: ContextMode,
}

pub(crate) struct ContextState {
    kind_counter: RwLock<u8>,
    conn: Arc<dyn ConnProvider>,
    encryption: Arc<dyn EncryptionProvider>,
    compression: Arc<dyn CompressionProvider>,
}

#[derive(Copy, Clone)]
pub(crate) enum ContextMode {
    Raw,
    Handle,
}

impl Context {
    pub(crate) fn new(conn: Arc<dyn ConnProvider>,
                      encryption: Arc<dyn EncryptionProvider>,
                      compression: Arc<dyn CompressionProvider>,
                      mode: ContextMode) -> Self {
        Context {
            state: Arc::new(ContextState {
                kind_counter: RwLock::new(1),
                conn,
                encryption,
                compression,
            }),
            mode,
        }
    }

    pub async fn get_kind_conn(&self) -> KindConn {
        *self.state.kind_counter.write().await += 1;
        let kind = *self.state.kind_counter.read().await - 1;
        KindConn::new(kind, self.mode, self.state.clone())
    }

    pub(crate) fn clone(&self, mode: ContextMode) -> Self {
        Context {
            state: self.state.clone(),
            mode,
        }
    }
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

    pub(crate) async fn write(&self, kind: u8, package: Vec<u8>,
                              mode: ContextMode) -> Result<(), WriteError<Vec<u8>>> {
        let frame = match mode {
            ContextMode::Raw => Frame::new(kind, package),
            ContextMode::Handle => {
                let package = self.encryption
                    .encrypt(package);
                let package = self.compression
                    .compress(package);
                Frame::new(kind, package)
            }
        };

        self.conn
            .write(frame)
            .await
            .map_err(|err| err.map(|frame| frame.get_data()))
    }

    pub(crate) fn local_addr(&self) -> SocketAddr {
        self.conn.local_addr()
    }

    pub(crate) fn peer_addr(&self) -> SocketAddr {
        self.conn.peer_addr()
    }

    pub(crate) async fn close(&self, code: u8) {
        self.conn.close(code).await
    }

    pub(crate) async fn is_close(&self) -> Option<u8> {
        self.conn.is_close().await
    }
}
