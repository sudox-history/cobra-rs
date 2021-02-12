use tokio::sync::RwLock;
use crate::transport::sync::WriteError;
use crate::manager::wrapper::Handler;
use std::sync::Arc;

pub struct Context<H: Handler> {
    handler: Arc<H>,
    kind_counter: RwLock<u8>,
}

impl<'a, H: Handler> Context<H> {
    pub(crate) async fn get_handler_conn(&self) -> HandlerConn<H> {
        unimplemented!()
    }
}

pub(crate) struct HandlerConn<H: Handler> {
    handler: Arc<H>,
    kind: u8,
}

impl<H: Handler> HandlerConn<H> {
    fn new(kind: u8) -> Self {
        unimplemented!()
    }

    pub(crate) async fn read(&self) -> Option<H::After> {
        unimplemented!()
    }

    pub(crate) async fn write(&self, package: H::After) -> Result<(), WriteError<H::After>> {
        unimplemented!()
    }
}
