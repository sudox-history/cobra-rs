use std::time::Duration;

use tokio::time::sleep;

use cobra_rs::builder::builder::Builder;
use cobra_rs::providers::default_ping_provider::DefaultPingProvider;
use cobra_rs::providers::tcp_conn_provider::TcpConnProvider;
use cobra_rs::transport::listener::Listener;

#[tokio::main]
async fn main() {
    tokio::join!(
        server(),
        client()
    );
}

async fn server() {
    let listener = Listener::listen("127.0.0.1:5000").await.unwrap();
    let conn = listener.accept().await.unwrap();
    let conn_provider = TcpConnProvider::from_tpc_conn(conn).await;
    let ping_provider = DefaultPingProvider::new(
        Duration::from_secs(6), Duration::from_secs(2));

    let conn = Builder::new()
        .set_conn(conn_provider)
        .set_ping(ping_provider)
        .run()
        .await
        .unwrap();
    println!("Server side built");

    assert_eq!(conn.read()
                   .await
                   .unwrap(), vec![1, 2, 3]);
    sleep(Duration::from_secs(10)).await;
    assert!(conn.write(vec![3, 2, 1])
        .await
        .is_ok());

    println!("Server side end");
}

async fn client() {
    let conn_provider = TcpConnProvider::new("127.0.0.1:5000").await.unwrap();
    let ping_provider = DefaultPingProvider::new(
        Duration::from_secs(6), Duration::from_secs(2));

    let conn = Builder::new()
        .set_conn(conn_provider)
        .set_ping(ping_provider)
        .run()
        .await
        .unwrap();
    println!("Client side built");

    assert!(conn.write(vec![1, 2, 3])
        .await
        .is_ok());
    assert_eq!(conn.read()
                   .await
                   .unwrap(), vec![3, 2, 1]);

    conn.close(1).await;
    println!("Client side end");
}
