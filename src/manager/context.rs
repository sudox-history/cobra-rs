use crate::manager::wrapper::{Middleware, Beginner};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::transport::sync::{Kind, WriteError};

pub struct Context<M: ?Sized> {
    pub middleware: Arc<dyn Middleware>,
    pub kind_counter: Arc<RwLock<u8>>,
}

impl<M: Beginner> Context<M> {
    pub fn new(beginner: M) -> Self {
        Context {
            middleware: Arc::new(beginner),
            kind_counter: Arc::new(RwLock::new(0)),
        }
    }
}

impl<M: Middleware> Context<M> {
    pub async fn get_kind_conn(&self) -> KindConn<M> {
        *self.kind_counter.write().await += 1;
        let kind = *self.kind_counter.read().await - 1;
        KindConn::new(kind, self.clone())
    }
}

impl<M: Middleware> Clone for Context<M> {
    fn clone(&self) -> Self {
        Context {
            middleware: self.middleware.clone(),
            kind_counter: self.kind_counter.clone(),
        }
    }
}

pub struct KindConn<M: Middleware> {
    kind: u8,
    context: Context<M>,
}

impl<M: Middleware> KindConn<M> {
    fn new(kind: u8 , context: Context<M>) -> Self {
        KindConn {
            kind,
            context,
        }
    }

    pub fn read(&self) -> Option<M::After> {
        unimplemented!()
    }

    pub fn write(&self, package: M::After) -> Result<(), WriteError<M::After>> {
        unimplemented!()
    }
}

impl<M: Middleware> Kind<u8> for KindConn<M> {
    fn kind(&self) -> u8 {
        self.kind
    }
}
