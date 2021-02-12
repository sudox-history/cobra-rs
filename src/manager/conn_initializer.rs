use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::transport::conn::Conn;
use crate::manager::wrapper::{Middleware, Beginner};
use crate::transport::frame::Frame;
use crate::transport::sync::WriteError;
use crate::manager::context::Context;

struct ConnInitializer {
    conn: Conn,
}

impl ConnInitializer {
    pub fn new(conn: Conn) -> Self {
        ConnInitializer {
            conn
        }
    }
}

#[async_trait]
impl Middleware for ConnInitializer {
    type Before = ();
    type After = Frame;

    async fn read(&self, package: Self::Before) -> Option<Self::After> {
        unimplemented!()
    }

    async fn write(&self, package: Self::After) -> Result<(), WriteError<Self::After>> {
        unimplemented!()
    }

    async fn run<M: Middleware + Send + Sync>(&self, context: Context<M>) {}
}

#[async_trait]
impl Beginner for ConnInitializer {
    async fn kind_write(&self, kind: u8, package: Frame) -> Result<(), WriteError<Frame>> {
        self.conn.write(package).await
    }

    async fn kind_read(&self, kind: u8) -> Option<Frame> {
        self.conn.read(kind).await
    }
}
