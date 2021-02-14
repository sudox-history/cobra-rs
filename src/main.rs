use cobra_rs::sync::{KindPool, Kind};

#[derive(Debug)]
struct Value {
    kind: u8,
}

impl Value {
    fn new(kind: u8) -> Self {
        Value { kind }
    }
}

impl Kind<u8> for Value {
    fn kind(&self) -> u8 {
        self.kind
    }
}

#[tokio::main]
async fn main() {
    let pool = KindPool::new();
    let pool2: KindPool<u8, Value> = pool.clone();
    let pool3: KindPool<u8, Value> = pool.clone();

    const KIND_1: u8 = 1;
    const KIND_2: u8 = 2;

    tokio::spawn(async move {
        loop {
            match pool2.read(KIND_1).await {
                Some(value) => {
                    println!("Received value with KIND_1: {:?}", *value);
                    value.accept();
                }
                None => break println!("Pool closed"),
            }
        }
    });

    tokio::spawn(async move {
        loop {
            match pool3.read(KIND_2).await {
                Some(value) => {
                    println!("Received value with KIND_2: {:?}", *value);
                    value.accept();
                }
                None => break println!("Pool closed"),
            }
        }
    });

    pool.write(Value::new(KIND_1)).await.unwrap();
    pool.write(Value::new(KIND_2)).await.unwrap();
    pool.close().await;
}
