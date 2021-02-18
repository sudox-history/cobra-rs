use std::sync::Arc;

use async_trait::async_trait;

use crate::builder::builder::{BuildError, CompressionProvider, EncryptionProvider, PingProvider};
use crate::builder::context::Context;

pub struct NilPing {}

impl NilPing {
    pub fn new() -> Arc<Self> {
        Arc::new(NilPing {})
    }
}

#[async_trait]
impl PingProvider for NilPing {
    async fn init(&self, _context: Context) {}
}

pub struct NilEncryption {}

impl NilEncryption {
    pub fn new() -> Arc<Self> {
        Arc::new(NilEncryption {})
    }
}

#[async_trait]
impl EncryptionProvider for NilEncryption {
    async fn init(&self, _context: Context) -> Result<(), BuildError> {
        Ok(())
    }

    fn encrypt(&self, frame: Vec<u8>) -> Vec<u8> {
        frame
    }

    fn decrypt(&self, frame: Vec<u8>) -> Vec<u8> {
        frame
    }
}

pub struct NilCompression {}

impl NilCompression {
    pub fn new() -> Arc<Self> {
        Arc::new(NilCompression {})
    }
}

#[async_trait]
impl CompressionProvider for NilCompression {
    async fn init(&self, _context: Context) {}

    fn compress(&self, frame: Vec<u8>) -> Vec<u8> {
        frame
    }

    fn decompress(&self, frame: Vec<u8>) -> Vec<u8> {
        frame
    }
}
