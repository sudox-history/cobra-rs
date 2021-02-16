use crate::manager::builder::{EncryptionManager, PingManager, CompressionManager, BuildError};
use crate::manager::context::Context;

use async_trait::async_trait;
use std::sync::Arc;

pub struct NilPing {}

impl NilPing {
    pub fn new() -> Arc<Self> {
        Arc::new(NilPing{})
    }
}

#[async_trait]
impl PingManager for NilPing {
    async fn init(&self, context: Context) { }
}

pub struct NilEncryption {}

impl NilEncryption {
    pub fn new() -> Arc<Self> {
        Arc::new(NilEncryption{})
    }
}

#[async_trait]
impl EncryptionManager for NilEncryption {
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
        Arc::new(NilCompression{})
    }
}

#[async_trait]
impl CompressionManager for NilCompression {
    async fn init(&self, context: Context) { }

    fn compress(&self, frame: Vec<u8>) -> Vec<u8> {
        frame
    }

    fn decompress(&self, frame: Vec<u8>) -> Vec<u8> {
        frame
    }
}
