use cobra_rs::transport::pool::{Pool, PoolOutput};
use tokio::time::{sleep, Duration};
use cobra_rs::transport::kind_pool::{KindPool, Kind};

#[derive(Debug)]
struct Value {
    a: i32
}

impl Value {
    fn create(a: i32) -> Self {
        Value { a }
    }
}

#[derive(Debug)]
struct TestFrame {
    key: u8,
    value: i32,
}

impl TestFrame {
    fn create(key: u8, value: i32) -> Self {
        TestFrame {
            key,
            value,
        }
    }
}

impl Kind<u8> for TestFrame {
    fn kind(&self) -> u8 {
        self.key
    }
}

#[tokio::main]
async fn main() {
    let read_pool = KindPool::new();

    const MAX: i32 = 32;

    for i in 0..MAX {
        let write_pool = read_pool.clone();
        tokio::spawn(async move {
            let package = TestFrame::create(0, 1);
            if write_pool.write(package).await.is_err() {
                panic!("Write return error")
            };
        });
    }

    for i in 0..MAX {
        assert_eq!(read_pool.read(0)
                       .await
                       .unwrap()
                       .value, 1);
    }

}
