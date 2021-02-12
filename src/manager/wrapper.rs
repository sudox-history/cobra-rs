use async_trait::async_trait;

use crate::transport::sync::WriteError;
use crate::manager::context::Context;

#[async_trait]
pub trait Handler {
    type Before;
    type After;

    async fn read(&self, package: Self::Before) -> Option<Self::After>;

    async fn write(&self, package: Self::After) -> Result<(), WriteError<Self::After>>;
}

#[async_trait]
pub trait Consumer {
    type Before;

    async fn run<H: Handler<After = Self::Before>>(context: Context<H>);
}

pub trait Middleware: Handler + Consumer {}

trait Wrapper {
    fn add_handler<H: Handler, O: Wrapper>(self, handler: H) -> O;

    fn add_consumer<E: Consumer, O: Wrapper>(self, executor: E) -> O;

    fn add_middleware<M: Middleware, O: Wrapper>(self, middleware: M) -> O;

    fn get_context<H: Handler>(&self) -> &Context<H>;
}

struct HandlerWrapper<H: Handler, W: Wrapper> {
    handler: H,
    previous: W,
    context: Context<H>,
}

impl<H: Handler, W: Wrapper> Wrapper for HandlerWrapper<H, W> {
    fn add_handler<T: Handler, O: Wrapper>(self, handler: T) -> O {
        unimplemented!()
    }

    fn add_consumer<E: Consumer, O: Wrapper>(self, executor: E) -> O {
        unimplemented!()
    }

    fn add_middleware<M: Middleware, O: Wrapper>(self, middleware: M) -> O {
        unimplemented!()
    }

    fn get_context(&self) -> &Context<H> {
        unimplemented!()
    }
}

struct ConsumerWrapper<C: Consumer, H: Handler, W: Wrapper> {
    consumer: C,
    previous: W,
    context: Context<H>,
}

impl<C: Consumer, H: Handler, W: Wrapper> Wrapper for ConsumerWrapper<C, H, W> {
    fn add_handler<V: Handler, O: Wrapper>(self, handler: V) -> O {
        unimplemented!()
    }

    fn add_consumer<T: Consumer, O: Wrapper>(self, executor: T) -> O {
        unimplemented!()
    }

    fn add_middleware<M: Middleware, O: Wrapper>(self, middleware: M) -> O {
        unimplemented!()
    }

    fn get_context(&self) -> &Context<H> {
        unimplemented!()
    }
}

struct MiddlewareWrapper<M: Middleware, W: Wrapper> {
    middleware: M,
    previous: W,
    context: Context<M>,
}

impl<M: Middleware, W: Wrapper> Wrapper for MiddlewareWrapper<M, W> {
    fn add_handler<H: Handler, O: Wrapper>(self, handler: H) -> O {
        unimplemented!()
    }

    fn add_consumer<E: Consumer, O: Wrapper>(self, executor: E) -> O {
        unimplemented!()
    }

    fn add_middleware<T: Middleware, O: Wrapper>(self, middleware: T) -> O {
        unimplemented!()
    }

    fn get_context(&self) -> &Context<M> {
        unimplemented!()
    }
}
