use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;

use crate::builder::context::{Context, ContextMode};
use crate::builder::empty_realisations::{NilCompression, NilEncryption, NilPing};
use crate::builder::kind_conn::KindConn;
use crate::sync::WriteError;
use crate::transport::frame::Frame;

#[async_trait]
pub trait ConnProvider: Send + Sync {
    async fn read(&self, kind: u8) -> Option<Frame>;

    async fn write(&self, frame: Frame) -> Result<(), WriteError<Frame>>;

    fn local_addr(&self) -> SocketAddr;

    fn peer_addr(&self) -> SocketAddr;

    async fn readable(&self) -> io::Result<()>;

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

pub enum BuildError {
    ConnNotSet,
    EncryptionInitFailed,
}

#[derive(Default)]
pub struct Builder {
    conn: Option<Arc<dyn ConnProvider>>,
    ping: Option<Arc<dyn PingProvider>>,
    encryption: Option<Arc<dyn EncryptionProvider>>,
    compression: Option<Arc<dyn CompressionProvider>>,
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
        self.ping = Some(Arc::new(ping));
        self
    }

    pub fn set_encryption<T: 'static + EncryptionProvider>(mut self, encryption: T) -> Self {
        self.encryption = Some(Arc::new(encryption));
        self
    }

    pub fn set_compression<T: 'static + CompressionProvider>(mut self, compression: T) -> Self {
        self.compression = Some(Arc::new(compression));
        self
    }

    pub async fn run(self) -> Result<KindConn, BuildError> {
        let conn = match self.conn {
            Some(conn) => conn,
            None => return Err(BuildError::ConnNotSet),
        };
        let ping = match self.ping {
            Some(ping) => ping,
            None => NilPing::new(),
        };
        let encryption = match self.encryption {
            Some(encryption) => encryption,
            None => NilEncryption::new(),
        };
        let compression = match self.compression {
            Some(compression) => compression,
            None => NilCompression::new(),
        };

        let context = Context::new(conn.clone(),
                                   encryption.clone(),
                                   compression,
                                   ContextMode::Handle);

        ping.init(context.clone(ContextMode::Handle)).await;
        encryption.init(context.clone(ContextMode::Raw)).await?;


        Ok(context.get_kind_conn().await)
    }
}
