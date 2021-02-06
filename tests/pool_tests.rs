#![allow(unused)]
use cobra_rs::transport::pool::*;

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
async fn one_read_two_write() {
    let read_pool = Pool::new();
    let write_pool_a = read_pool.clone();
    let write_pool_b = read_pool.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_A, 0);
        let package_b = TestFrame::create(KIND_A, 1);
        write_pool_a.write(package_a).await;
        write_pool_a.write(package_b).await;
    });

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_B, 2);
        let package_b = TestFrame::create(KIND_B, 3);
        write_pool_b.write(package_a).await;
        write_pool_b.write(package_b).await;
    });

    assert_eq!(read_pool.read(KIND_A)
                   .await
                   .unwrap()
                   .value, 0);
    assert_eq!(read_pool.read(KIND_A)
                   .await
                   .unwrap()
                   .value, 1);
    assert_eq!(read_pool.read(KIND_B)
                   .await
                   .unwrap()
                   .value, 2);
    assert_eq!(read_pool.read(KIND_B)
                   .await
                   .unwrap()
                   .value, 3);
}

#[tokio::test]
async fn two_read_one_write() {
    let read_pool_a = Pool::new();
    let read_pool_b = read_pool_a.clone();
    let write_pool = read_pool_a.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_A, 0);
        let package_b = TestFrame::create(KIND_B, 1);
        write_pool.write(package_a).await;
        write_pool.write(package_b).await;
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
async fn two_read_two_write() {
    let read_pool_a = Pool::new();
    let read_pool_b = read_pool_a.clone();
    let write_pool_a = read_pool_a.clone();
    let write_pool_b = read_pool_b.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        let package_a = TestFrame::create(KIND_A, 0);
        write_pool_a.write(package_a).await;
    });

    tokio::spawn(async move {
        let package_b = TestFrame::create(KIND_B, 1);
        write_pool_b.write(package_b).await;
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
async fn stress_test() {
    let read_pool = Pool::new();

    const KIND_A: u8 = 0;

    for i in 0..100000 {
        let write_pool = read_pool.clone();
        tokio::spawn(async move {
            let package = TestFrame::create(KIND_A, i);
            write_pool.write(package).await;
        });
    }

    for i in 0..100000 {
        assert_eq!(read_pool.read(KIND_A)
                       .await
                       .unwrap()
                       .value, i);
    }
}

#[tokio::test]
#[should_panic]
async fn write_err() {
    let close_pool: Pool<u8, TestFrame> = Pool::new();
    let write_pool = close_pool.clone();

    const KIND_A: u8 = 0;

    close_pool.close().await;
    let package = TestFrame::create(KIND_A, 0);
    match write_pool.write(package).await {
        Ok(_) => {},
        Err(_) => {
            panic!("Write err")
        }
    }
}

#[tokio::test]
#[should_panic]
async fn err_after_write() {
    let read_pool = Pool::new();
    let write_pool = read_pool.clone();

    const KIND_A: u8 = 0;

    tokio::spawn(async move {
        let package = TestFrame::create(KIND_A, 0);
        write_pool.write(package).await;
        write_pool.close().await;
    });

    assert_eq!(read_pool.read(KIND_A)
                   .await
                   .unwrap()
                   .value, 0);
    read_pool.read(KIND_A).await.unwrap();
}

#[tokio::test]
async fn read_any() {
    let read_any_pool = Pool::new();
    let write_any_pool_a = read_any_pool.clone();
    let write_any_pool_b = read_any_pool.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        let package = TestFrame::create(KIND_A, 0);
        write_any_pool_a.write(package).await;
    });

    tokio::spawn(async move {
        let package = TestFrame::create(KIND_B, 0);
        write_any_pool_b.write(package).await;
    });

    assert_eq!(read_any_pool.read_any()
                   .await
                   .unwrap()
                   .value, 0);
    assert_eq!(read_any_pool.read_any()
                   .await
                   .unwrap()
                   .value, 0);
}

#[tokio::test]
async fn data_order() {
    let read_pool = Pool::new();
    let write_pool_a = read_pool.clone();
    let write_pool_b = read_pool.clone();

    const KIND_A: u8 = 0;
    const KIND_B: u8 = 1;

    tokio::spawn(async move {
        for i in 0..5 {
            let package = TestFrame::create(KIND_A, i);
            write_pool_a.write(package).await;
        }
    });

    tokio::spawn(async move {
        for i in 0..5 {
            let package = TestFrame::create(KIND_B, i);
            write_pool_b.write(package).await;
        }
    });

    for i in 0..5 {
        assert_eq!(read_pool.read(KIND_A)
            .await
            .unwrap()
            .value, i);
        assert_eq!(read_pool.read(KIND_B)
            .await
            .unwrap()
            .value, i);
    }
}