use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;

use crate::builder::context::{Context, ContextMode};
use crate::builder::empty_realisations::EmptyRealisation;
use crate::builder::kind_conn::KindConn;
use crate::mem::Frame;
use crate::sync::WriteError;
use std::io;

#[async_trait]
pub trait ConnProvider: Send + Sync {
    async fn read(&self, kind: u8) -> Option<Frame>;

    async fn write(&self, frame: Frame) -> Result<(), WriteError<Frame>>;

    fn local_addr(&self) -> io::Result<SocketAddr>;

    fn peer_addr(&self) -> io::Result<SocketAddr>;

    async fn readable(&self);

    async fn close(&self, code: u8);

    // Return None if conn is able, else return close code
    async fn is_close(&self) -> Option<u8>;
}

#[async_trait]
pub trait PingProvider: Send + Sync {
    async fn init(&self, context: Context);
}

#[async_trait]
pub trait EncryptionProvider: Send + Sync {
    async fn init(&self, context: Context) -> Result<(), BuildError>;

    fn encrypt(&self, frame: Vec<u8>) -> Vec<u8>;

    fn decrypt(&self, frame: Vec<u8>) -> Vec<u8>;
}

#[async_trait]
pub trait CompressionProvider: Send + Sync {
    async fn init(&self, context: Context);

    fn compress(&self, frame: Vec<u8>) -> Vec<u8>;

    fn decompress(&self, frame: Vec<u8>) -> Vec<u8>;
}

#[derive(Debug)]
pub enum BuildError {
    ConnNotSet,
    EncryptionInitFailed,
}

pub struct Builder {
    conn: Option<Arc<dyn ConnProvider>>,
    ping: Arc<dyn PingProvider>,
    encryption: Arc<dyn EncryptionProvider>,
    compression: Arc<dyn CompressionProvider>,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_conn<T: 'static + ConnProvider>(mut self, conn: T) -> Self {
        self.conn = Some(Arc::new(conn));
        self
    }

    pub fn set_ping<T: 'static + PingProvider>(mut self, ping: T) -> Self {
        self.ping = Arc::new(ping);
        self
    }

    pub fn set_encryption<T: 'static + EncryptionProvider>(mut self, encryption: T) -> Self {
        self.encryption = Arc::new(encryption);
        self
    }

    pub fn set_compression<T: 'static + CompressionProvider>(mut self, compression: T) -> Self {
        self.compression = Arc::new(compression);
        self
    }

    pub async fn run(self) -> Result<KindConn, BuildError> {
        let conn = match self.conn {
            Some(conn) => conn,
            None => return Err(BuildError::ConnNotSet),
        };
        let context = Context::new(conn.clone(),
                                   self.encryption.clone(),
                                   self.compression,
                                   ContextMode::Handle);

        self.ping.init(context.clone(ContextMode::Raw)).await;
        self.encryption.init(context.clone(ContextMode::Raw)).await?;

        Ok(context.get_kind_conn().await)
    }
}

impl Default for Builder {
    fn default() -> Self {
        let empty_realisation = EmptyRealisation::new();
        Builder {
            conn: None,
            ping: empty_realisation.clone(),
            encryption: empty_realisation.clone(),
            compression: empty_realisation.clone(),
        }
    }
}
