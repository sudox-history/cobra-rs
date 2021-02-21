use std::sync::Arc;

use tokio::sync::RwLock;

use crate::builder::builder::{CompressionProvider, ConnProvider, EncryptionProvider};
use crate::builder::kind_conn::KindConn;

pub(crate) struct ContextState {
    kind_counter: RwLock<u8>,
    pub(crate) conn: Arc<dyn ConnProvider>,
    pub(crate) encryption: Arc<dyn EncryptionProvider>,
    pub(crate) compression: Arc<dyn CompressionProvider>,
}

#[derive(Copy, Clone)]
pub(crate) enum ContextMode {
    Raw,
    Handle,
}

pub struct Context {
    state: Arc<ContextState>,
    mode: ContextMode,
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
