use cobra_rs::sync::{KindPool, Kind, Pool, WriteError};
use std::fs::read;

#[derive(Debug)]
struct Value {
    kind: u8,
}

impl Value {
    fn new(kind: u8) -> Self {
        Value { kind }
    }
}

impl Kind<u8> for Value {
    fn kind(&self) -> u8 {
        self.kind
    }
}

#[tokio::main]
async fn main() {
    let read_pool: Pool<i32> = Pool::new();
    let write_pool: Pool<i32> = read_pool.clone();

    tokio::spawn(async move {
        write_pool.close().await;
    });

    assert!(read_pool.read().await.is_none());
}


