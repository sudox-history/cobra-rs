use std::time::Duration;

use cobra_rs::builder::builder::Builder;
use cobra_rs::providers::default_ping_provider::DefaultPingProvider;
use cobra_rs::transport::tcp::{Conn, Listener};

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
    let ping_provider = DefaultPingProvider::new(
        Duration::from_secs(6), Duration::from_secs(2));

    let conn = Builder::new()
        .set_conn(conn)
        .set_ping(ping_provider)
        .run()
        .await
        .unwrap();
    println!("Server side built");

    let frame = conn.read()
        .await
        .unwrap();
    assert_eq!(frame, vec![1, 2, 3]);

    let frame = vec![3, 2, 1];
    assert!(conn.write(frame)
        .await
        .is_ok());

    println!("Server side end");
}

async fn client() {
    let conn_provider = Conn::connect("127.0.0.1:5000").await.unwrap();
    let ping_provider = DefaultPingProvider::new(
        Duration::from_secs(6), Duration::from_secs(2));

    let conn = Builder::new()
        .set_conn(conn_provider)
        .set_ping(ping_provider)
        .run()
        .await
        .unwrap();
    println!("Client side built");

    let frame = vec![1, 2, 3];
    assert!(conn.write(frame)
        .await
        .is_ok());

    let frame = conn.read()
        .await
        .unwrap();
    assert_eq!(frame, vec![3, 2, 1]);

    println!("Client side end");
}
