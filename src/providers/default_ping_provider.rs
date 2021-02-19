use std::time::Duration;

use async_trait::async_trait;
use tokio::time::timeout;

use crate::builder::builder::PingProvider;
use crate::builder::context::Context;
use crate::builder::kind_conn::KindConn;

pub struct DefaultPingProvider {
    long_duration: Duration,
    short_duration: Duration,
}

#[async_trait]
impl PingProvider for DefaultPingProvider {
    async fn init(&self, context: Context) {
        let conn = context.get_kind_conn().await;
        tokio::spawn(ping_loop(self.long_duration,
                               self.short_duration, conn));
    }
}

impl DefaultPingProvider {
    pub fn new(long_duration: Duration, short_duration: Duration) -> Self {
        DefaultPingProvider {
            long_duration,
            short_duration,
        }
    }
}

async fn ping_loop(long_duration: Duration, short_duration: Duration, conn: KindConn) {
    loop {
        tokio::select! {
            res = ping(long_duration, short_duration, &conn) => {
                if res.is_err() {
                    break;
                }
            },
            res = conn.readable() => {
                if res.is_err() {
                    break;
                }
            }
        }
    }
}

async fn ping(long_duration: Duration, short_duration: Duration, conn: &KindConn) -> Result<(), ()> {
    if long_timeout(long_duration, conn).await.is_ok() {
        Ok(())
    } else {
        short_timeout(short_duration, conn).await
    }
}

async fn long_timeout(long_duration: Duration, conn: &KindConn) -> Result<(), ()>
{
    match timeout(long_duration, conn.read()).await {
        Ok(res) => handle_read(res, conn).await,
        Err(_) => write_ping(conn).await,
    }
}

async fn short_timeout(short_duration: Duration,
                       conn: &KindConn) -> Result<(), ()> {
    match timeout(short_duration, conn.read()).await {
        Ok(res) => handle_read(res, conn).await,
        Err(_) => {
            conn.close(5).await;
            Err(())
        }
    }
}

async fn handle_read(res: Option<Vec<u8>>, conn: &KindConn) -> Result<(), ()> {
    match res {
        Some(_) => write_ping(&conn).await,
        None => Err(()),
    }
}

async fn write_ping(conn: &KindConn) -> Result<(), ()> {
    conn.write(vec![])
        .await
        .map_err(|_| ())
}
