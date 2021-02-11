use cobra_rs::transport::sync::{Pool};

#[derive(Debug)]
struct Value {
    a: i32,
}

impl Value {
    fn new(a: i32) -> Self {
        Value { a }
    }
}

#[tokio::main]
async fn main() {
    let pool = Pool::new();
    let pool2: Pool<i32> = pool.clone();

    tokio::spawn(async move {
        loop {
            match pool2.read().await {
                Some(value) => {
                    println!("Received value: {:?}", *value);
                    value.accept();
                }
                None => {
                    println!("Pool closed");
                    break;
                }
            }
        }
    });

    pool.write(12).await.unwrap();
    pool.close();
}
