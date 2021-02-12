use async_trait::async_trait;

use crate::transport::sync::WriteError;
use crate::manager::context::Context;

#[async_trait]
pub trait Middleware {
    type Before;
    type After;

    async fn read(&self, package: Self::Before) -> Option<Self::After>;

    async fn write(&self, package: Self::After) -> Result<(), WriteError<Self::After>>;

    async fn run<M: Middleware + Sync>(&self, context: Context<M>);
}

#[async_trait]
pub trait Beginner: Middleware<Before = ()> {
    async fn kind_write(&self, kind: u8, package: Self::After) -> Result<(), WriteError<Self::After>>;

    async fn kind_read(&self, kind: u8) -> Option<Self::After>;
}

#[async_trait]
pub trait Consumer {
    type Incoming;

    async fn run<M: Middleware>(self, context: Context<M>);
}

pub struct Wrapper<M: Middleware> {
    context: Context<M>,
}

impl<M: Beginner> Wrapper<M> {
    pub fn new(beginner: M) -> Self {
        Wrapper {
            context: Context::new(beginner),
        }
    }
}

impl<M: Middleware> Wrapper<M> {
    fn add_middleware<O: Middleware<Before = M::After>>(self, middleware: O) -> Wrapper<O> {
        unimplemented!()
    }

    fn add_consumer<O: Consumer<Incoming = M::After>>(self, consumer: O) -> Wrapper<M> {
        unimplemented!()
    }
}
