use cobra_rs::transport::pool::Pool;
use cobra_rs::transport::frame::Frame;
use tokio::time;

const PING_KIND: u8 = 1;
const HANDSHAKE_KIND: u8 = 2;

#[tokio::main]
async fn main() {
    let pool = Pool::new();
    let pool2 = pool.clone();
    let pool3 = pool.clone();

    tokio::spawn(async move {
        loop {
            let data = pool2.read(PING_KIND).await;

            match data {
                Some(data) => println!("Received PING frame {:?}", data),
                None => {
                    println!("Pool closed");
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        loop {
            let data = pool3.read(HANDSHAKE_KIND).await;

            match data {
                Some(data) => println!("Received HANDSHAKE frame {:?}", data),
                None => {
                    println!("Pool closed");
                    break;
                }
            }
        }
    });

    let ping_frame = Frame::create(PING_KIND, vec![]);
    let handshake_frame = Frame::create(HANDSHAKE_KIND, vec![1, 2, 3]);

    pool.write(ping_frame).await.unwrap();
    time::sleep(time::Duration::from_secs(4)).await;

    pool.write(handshake_frame).await.unwrap();
    time::sleep(time::Duration::from_secs(4)).await;

    pool.close().await;
}