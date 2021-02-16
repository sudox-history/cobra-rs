use std::io;

use async_trait::async_trait;

use crate::manager::context::{Context, KindConn};
use crate::sync::WriteError;
use crate::transport::frame::Frame;
use std::sync::Arc;
use crate::manager::empty_realisations::{NilPing, NilEncryption, NilCompression};

#[async_trait]
pub trait ConnManager: Send + Sync {
    async fn connect(&self) -> io::Result<()>;

    async fn read(&self, kind: u8) -> Option<Frame>;

    async fn write(&self, frame: Frame) -> Result<(), WriteError<Frame>>;
}

#[async_trait]
pub trait PingManager: Send + Sync {
    async fn init(&self, context: Context);
}

#[async_trait]
pub trait EncryptionManager: Send + Sync {
    async fn init(&self, context: Context) -> Result<(), BuildError>;

    fn encrypt(&self, frame: Vec<u8>) -> Vec<u8>;

    fn decrypt(&self, frame: Vec<u8>) -> Vec<u8>;
}

#[async_trait]
pub trait CompressionManager: Send + Sync {
    async fn init(&self, context: Context);

    fn compress(&self, frame: Vec<u8>) -> Vec<u8>;

    fn decompress(&self, frame: Vec<u8>) -> Vec<u8>;
}

pub enum BuildError {
    ConnNotSet,
    ConnectionFailed,
    EncryptionInitFailed,
}

pub struct Builder {
    conn: Option<Arc<dyn ConnManager>>,
    ping: Option<Arc<dyn PingManager>>,
    encryption: Option<Arc<dyn EncryptionManager>>,
    compression: Option<Arc<dyn CompressionManager>>,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_conn<T: 'static +  ConnManager>(mut self, conn: T) -> Self {
        self.conn = Some(Arc::new(conn));
        self
    }

    pub fn set_ping<T: 'static +  PingManager>(mut self, ping: T) -> Self {
        self.ping = Some(Arc::new(ping));
        self
    }

    pub fn set_encryption<T: 'static +  EncryptionManager >(mut self, encryption: T) -> Self {
        self.encryption = Some(Arc::new(encryption));
        self
    }

    pub fn set_compression<T: 'static +  CompressionManager>(mut self, compression: T) -> Self {
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
                                   ping.clone(),
                                   encryption.clone(),
                                   compression);

        conn.connect().await.map_err(|_| BuildError::ConnectionFailed)?;
        encryption.init(context.clone()).await?;

        let ping_context = context.clone();
        tokio::spawn(async move {
            ping.init(ping_context).await;
        });

        Ok(context.get_kind_conn().await)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            conn: None,
            ping: None,
            encryption: None,
            compression: None,
        }
    }
}
