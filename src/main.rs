use cobra_rs::transport::pool::PoolAny;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct Value {
    a: i32
}

impl Value {
    fn create(a: i32) -> Self {
        Value { a }
    }
}

#[tokio::main]
async fn main() {
    let pool = PoolAny::new();
    let pool2 = pool.clone();

    tokio::spawn(async move {
        loop {
            let data = pool2.read_any().await;

            match data {
                Some(data) => println!("Received value {:?}", data),
                None => {
                    println!("Pool closed");
                    break;
                }
            }
        }
    });

    let value_a = Value::create(1);
    let value_b = Value::create(2);

    pool.write(value_a).await.unwrap();
    sleep(Duration::from_secs(4)).await;

    pool.write(value_b).await.unwrap();
    sleep(Duration::from_secs(4)).await;

    pool.close().await;
    sleep(Duration::from_secs(1)).await;
}
