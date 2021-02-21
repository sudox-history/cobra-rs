use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::time;

use cobra_rs::sync::{Pool, WriteError};

#[tokio::test]
async fn one_read_one_write() {
    let read_pool = Pool::new();
    let write_pool = read_pool.clone();

    tokio::spawn(async move {
        write_pool.write(1).await.unwrap();
    });

    assert_eq!(read_pool.read()
                   .await
                   .unwrap()
                   .accept(), 1);
}

#[tokio::test]
async fn one_read_multiple_write() {
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
async fn multiple_read_one_write() {
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
async fn multiple_read_multiple_write() {
    let write_pool_a: Pool<usize> = Pool::new();
    let write_pool_b: Pool<usize> = write_pool_a.clone();
    let read_pool_a = write_pool_a.clone();
    let read_pool_b = write_pool_a.clone();

    let semaphore = Arc::new(Semaphore::new(0));
    let semaphore_a = semaphore.clone();
    let semaphore_b = semaphore.clone();

    tokio::spawn(async move {
        semaphore_a.add_permits(read_pool_a.read().await.unwrap().accept());
    });

    tokio::spawn(async move {
        semaphore_b.add_permits(read_pool_b.read().await.unwrap().accept());
    });

    tokio::spawn(async move {
        write_pool_a.write(1).await.unwrap();
    });

    tokio::spawn(async move {
        write_pool_b.write(2).await.unwrap();
    });

    assert!(semaphore.acquire_many(3).await.is_ok());
}


#[tokio::test]
async fn implicit_accept_test() {
    let read_pool = Pool::new();
    let write_pool = read_pool.clone();

    tokio::spawn(async move {
        read_pool.read()
            .await
            .unwrap();
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

    match write_pool.write(1).await.unwrap_err() {
        WriteError::Rejected(value) => assert_eq!(value, 1),
        _ => panic!("wrong write error returned"),
    }
}

#[tokio::test]
async fn read_after_close_test() {
    let read_pool: Pool<i32> = Pool::new();
    read_pool.close().await;

    assert!(read_pool.read().await.is_none());
}

#[tokio::test]
async fn read_before_close_test() {
    let read_pool: Pool<i32> = Pool::new();
    let write_pool: Pool<i32> = read_pool.clone();

    tokio::spawn(async move {
        time::sleep(time::Duration::from_millis(100)).await;
        write_pool.close().await;
    });

    assert!(read_pool.read().await.is_none());
}

#[tokio::test]
async fn write_after_close_test() {
    let write_pool: Pool<i32> = Pool::new();
    write_pool.close().await;

    match write_pool.write(1).await.unwrap_err() {
        WriteError::Closed(value) => assert_eq!(value, 1),
        _ => panic!("wrong write error returned")
    };
}

#[tokio::test]
async fn write_before_close_test() {
    let read_pool: Pool<i32> = Pool::new();
    let write_pool: Pool<i32> = read_pool.clone();

    tokio::spawn(async move {
        time::sleep(time::Duration::from_millis(100)).await;
        read_pool.close().await;
    });

    match write_pool.write(1).await.unwrap_err() {
        WriteError::Closed(value) => assert_eq!(value, 1),
        _ => panic!("wrong write error returned")
    }
}

#[tokio::test]
async fn accept_after_close_test() {
    let read_pool: Pool<i32> = Pool::new();
    let write_pool: Pool<i32> = read_pool.clone();

    tokio::spawn(async move {
        let value = read_pool.read().await.unwrap();
        read_pool.close().await;

        value.accept();
    });

    assert!(write_pool.write(1).await.is_ok());
}

#[tokio::test]
async fn reject_after_close_test() {
    let read_pool: Pool<i32> = Pool::new();
    let write_pool: Pool<i32> = read_pool.clone();

    tokio::spawn(async move {
        let value = read_pool.read().await.unwrap();
        read_pool.close().await;

        value.reject().await;
    });

    match write_pool.write(1).await.unwrap_err() {
        WriteError::Rejected(value) => assert_eq!(value, 1),
        _ => panic!("wrong write error returned")
    }
}

#[tokio::test]
async fn stress_test() {
    let read_pool: Pool<i32> = Pool::new();
    let write_pool: Pool<i32> = read_pool.clone();

    tokio::spawn(async move {
        for i in 0..30000 {
            write_pool.write(i).await.unwrap();
        }
    });

    for i in 0..30000 {
        assert_eq!(read_pool.read().await.unwrap().accept(), i);
    }
}
