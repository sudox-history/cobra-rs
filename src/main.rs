use cobra_rs::transport::kind_pool::{KindPool, Kind};

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
    let close_pool: KindPool<u8, TestFrame> = KindPool::new();
    let write_pool = close_pool.clone();

    const KIND_A: u8 = 0;

    close_pool.close().await;
    let package = TestFrame::create(KIND_A, 0);
    assert!(write_pool.write(package).await.is_err());
}
