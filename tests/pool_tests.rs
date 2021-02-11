use cobra_rs::transport::sync::Pool;
use tokio::sync::Semaphore;
use std::sync::Arc;

#[tokio::test]
async fn one_read_two_write() {
    let read_pool = Pool::new();
    let write_pool_a = read_pool.clone();
    let write_pool_b = read_pool.clone();

    tokio::spawn(async move {
        write_pool_a.write(1).await.unwrap();
    });

    tokio::spawn(async move {
        write_pool_b.write(1).await.unwrap();
    });

    assert_eq!(read_pool.read()
                   .await
                   .unwrap()
                   .accept(), 1);
    assert_eq!(read_pool.read()
                   .await
                   .unwrap()
                   .accept(), 1);
}

#[tokio::test]
async fn two_read_one_write() {
    let write_pool: Pool<usize> = Pool::new();
    let read_pool_a = write_pool.clone();
    let read_pool_b = write_pool.clone();

    let semaphore = Arc::new(Semaphore::new(0));
    let semaphore_a = semaphore.clone();
    let semaphore_b = semaphore.clone();

    tokio::spawn(async move {
        let val = read_pool_a.read().await.unwrap().accept();
        semaphore_a.add_permits(val);
    });

    tokio::spawn(async move {
        let val = read_pool_b.read().await.unwrap().accept();
        semaphore_b.add_permits(val);
    });

    assert!(write_pool.write(1).await.is_ok());
    assert!(write_pool.write(2).await.is_ok());

    assert!(semaphore.acquire_many(3).await.is_ok());
}

#[tokio::test]
async fn difference_data_test() {
    let read_pool = Pool::new();

    for i in 0..100 {
        let write_pool = read_pool.clone();
        tokio::spawn(async move {
            assert!(write_pool.write(i).await.is_ok());
        });
    }

    let mut count = 0;
    for _ in 0..100 {
        count += read_pool.read().await.unwrap().accept();
    }

    assert_eq!(count, 100 * 99 / 2)
}

#[tokio::test]
async fn implicit_accept_test() {
    let read_pool = Pool::new();
    let write_pool = read_pool.clone();

    tokio::spawn(async move {
        let value = read_pool.read()
            .await
            .unwrap();
        assert_eq!(*value, 1);
    });

    assert!(write_pool.write(1).await.is_ok());
}

#[tokio::test]
async fn reject_test() {
    let read_pool: Pool<i32> = Pool::new();
    let write_pool: Pool<i32> = read_pool.clone();

    tokio::spawn(async move {
        read_pool.read().await.unwrap().reject().await;
    });

    assert!(write_pool.write(1).await.is_err());
}

#[tokio::test]
async fn close_test() {
    let read_pool: Pool<i32> = Pool::new();
    let write_pool: Pool<i32> = read_pool.clone();

    tokio::spawn(async move {
        assert!(write_pool.write(1).await.is_err());
    });

    read_pool.close();
    assert!(read_pool.read().await.is_none());
}
