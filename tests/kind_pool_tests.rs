use cobra_rs::sync::{Kind, KindPool};

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

#[tokio::test]
async fn one_read_one_write() {
    let read_pool = KindPool::new();
    let write_pool = read_pool.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_A, 0);
        let package_b = TestFrame::create(KIND_B, 1);
        write_pool.write(package_a).await.unwrap();
        write_pool.write(package_b).await.unwrap();
    });

    assert_eq!(read_pool.read(KIND_A)
                   .await
                   .unwrap()
                   .accept()
                   .value, 0);

    assert_eq!(read_pool.read(KIND_B)
                   .await
                   .unwrap()
                   .accept()
                   .value, 1);
}

#[tokio::test]
async fn one_read_multiple_write() {
    let read_pool = KindPool::new();
    let write_pool_a = read_pool.clone();
    let write_pool_b = read_pool.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_A, 0);
        let package_b = TestFrame::create(KIND_A, 1);
        write_pool_a.write(package_a).await.unwrap();
        write_pool_a.write(package_b).await.unwrap();
    });

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_B, 2);
        let package_b = TestFrame::create(KIND_B, 3);
        write_pool_b.write(package_a).await.unwrap();
        write_pool_b.write(package_b).await.unwrap();
    });

    assert_eq!(read_pool.read(KIND_A)
                   .await
                   .unwrap()
                   .accept()
                   .value, 0);
    assert_eq!(read_pool.read(KIND_A)
                   .await
                   .unwrap()
                   .accept()
                   .value, 1);
    assert_eq!(read_pool.read(KIND_B)
                   .await
                   .unwrap()
                   .accept()
                   .value, 2);
    assert_eq!(read_pool.read(KIND_B)
                   .await
                   .unwrap()
                   .accept()
                   .value, 3);
}

#[tokio::test]
async fn multiple_read_one_write() {
    let read_pool_a = KindPool::new();
    let read_pool_b = read_pool_a.clone();
    let write_pool = read_pool_a.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_A, 0);
        let package_b = TestFrame::create(KIND_B, 1);
        write_pool.write(package_a).await.unwrap();
        write_pool.write(package_b).await.unwrap();
    });

    assert_eq!(read_pool_a.read(KIND_A)
                   .await
                   .unwrap()
                   .accept()
                   .value, 0);
    assert_eq!(read_pool_b.read(KIND_B)
                   .await
                   .unwrap()
                   .accept()
                   .value, 1);
}

#[tokio::test]
async fn multiple_read_multiple_write() {
    let read_pool_a = KindPool::new();
    let read_pool_b = read_pool_a.clone();
    let write_pool_a = read_pool_a.clone();
    let write_pool_b = read_pool_b.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_A, 0);
        write_pool_a.write(package_a).await.unwrap();
    });

    tokio::spawn(async move {
        let package_b = TestFrame::create(KIND_B, 1);
        write_pool_b.write(package_b).await.unwrap();
    });

    assert_eq!(read_pool_a.read(KIND_A)
                   .await
                   .unwrap()
                   .value, 0);
    assert_eq!(read_pool_b.read(KIND_B)
                   .await
                   .unwrap()
                   .value, 1);
}


#[tokio::test]
async fn write_after_close() {
    let close_pool: KindPool<u8, TestFrame> = KindPool::new();
    let write_pool = close_pool.clone();

    const KIND_A: u8 = 0;

    close_pool.close().await;
    let package = TestFrame::create(KIND_A, 0);
    assert!(write_pool.write(package).await.is_err())
}

#[tokio::test]
async fn read_after_close() {
    let read_pool: KindPool<u8, TestFrame> = KindPool::new();
    let write_pool = read_pool.clone();

    const KIND_A: u8 = 0;

    tokio::spawn(async move {
        write_pool.close().await;
    });

    assert!(read_pool.read(KIND_A).await.is_none());
}

#[tokio::test]
async fn data_order() {
    let read_pool = KindPool::new();
    let write_pool_a = read_pool.clone();
    let write_pool_b = read_pool.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        for i in 0..5 {
            let package = TestFrame::create(KIND_A, i);
            write_pool_a.write(package).await.unwrap();
        }
    });

    tokio::spawn(async move {
        for i in 0..5 {
            let package = TestFrame::create(KIND_B, i);
            write_pool_b.write(package).await.unwrap();
        }
    });

    for i in 0..5 {
        assert_eq!(read_pool.read(KIND_A)
                       .await
                       .unwrap()
                       .accept()
                       .value, i);
        assert_eq!(read_pool.read(KIND_B)
                       .await
                       .unwrap()
                       .accept()
                       .value, i);
    }
}

#[tokio::test]
async fn stress_test() {
    let read_pool = KindPool::new();
    let write_pool = read_pool.clone();

    tokio::spawn(async move {
        for i in 0..10000 {
            let package = TestFrame::create(0, i);
            assert!(write_pool.write(package).await.is_ok());
        }
    });

    for i in 0..10000 {
        assert_eq!(read_pool.read(0).await.unwrap().accept().value, i);
    }
}
