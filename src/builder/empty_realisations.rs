use std::sync::Arc;

use async_trait::async_trait;

use crate::builder::builder::{BuildError, CompressionProvider, EncryptionProvider, PingProvider};
use crate::builder::context::Context;

pub struct EmptyRealisation {}

impl EmptyRealisation {
    pub fn new() -> Arc<Self> {
        Arc::new(EmptyRealisation {})
    }
}

#[async_trait]
impl PingProvider for EmptyRealisation {
    async fn init(&self, _context: Context) {}
}

#[async_trait]
impl EncryptionProvider for EmptyRealisation {
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

#[async_trait]
impl CompressionProvider for EmptyRealisation {
    async fn init(&self, _context: Context) {}

    fn compress(&self, frame: Vec<u8>) -> Vec<u8> {
        frame
    }

    fn decompress(&self, frame: Vec<u8>) -> Vec<u8> {
        frame
    }
}
