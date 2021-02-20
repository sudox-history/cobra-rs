use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::builder::builder::PingProvider;
use crate::builder::context::Context;
use crate::builder::kind_conn::close_code::PING_TIMEOUT;
use crate::builder::kind_conn::KindConn;

pub struct DefaultPingProvider {
    long_duration: Duration,
    short_duration: Duration,
}

#[async_trait]
impl PingProvider for DefaultPingProvider {
    async fn init(&self, context: Context) {
        let conn = Arc::new(context.get_kind_conn().await);
        let alive = Arc::new(RwLock::new(true));

        tokio::spawn(
            DefaultPingProvider::read_loop(conn.clone(), alive.clone())
        );
        tokio::spawn(
            DefaultPingProvider::ping_loop(self.long_duration, self.short_duration, conn, alive)
        );
    }
}

impl DefaultPingProvider {
    pub fn new(long_duration: Duration, short_duration: Duration) -> Self {
        DefaultPingProvider {
            long_duration,
            short_duration,
        }
    }

    async fn ping_loop(long_duration: Duration,
                       short_duration: Duration,
                       conn: Arc<KindConn>,
                       alive: Arc<RwLock<bool>>) {
        loop {
            // Если ошибка - то прошел таймаут и не было принято пакетов
            if timeout(long_duration, conn.readable()).await.is_err() {
                *alive.write().await = false;
                if DefaultPingProvider::write_ping(&conn).await.is_err() {
                    break;
                };

                if timeout(short_duration, conn.readable()).await.is_err()
                    && !(*alive.read().await) {
                    conn.close(PING_TIMEOUT).await;
                }
            }
        }
    }

    async fn read_loop(conn: Arc<KindConn>, alive: Arc<RwLock<bool>>) {
        while conn.read().await.is_some() {
            if *alive.read().await {
                if DefaultPingProvider::write_ping(&conn).await.is_err() {
                    break;
                }
            } else {
                *alive.write().await = false;
            }
        }
    }

    async fn write_ping(conn: &KindConn) -> Result<(), ()> {
        conn.write(vec![])
            .await
            .map_err(|_| ())
    }
}
