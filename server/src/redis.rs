use crate::app::{AppError, AppResult};
use mobc_redis::{
    mobc::{Connection, Pool},
    redis::Client,
    RedisConnectionManager,
};
use std::time;

pub struct RedisPool {
    addr: String,
    pool: Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub fn new(addr: String) -> Self {
        let pool = Pool::builder()
            .max_open(20)
            .build(RedisConnectionManager::new(
                Client::open(addr.clone()).unwrap(),
            ));
        Self { addr, pool }
    }

    pub async fn get_connection(&self) -> AppResult<Connection<RedisConnectionManager>> {
        trace!("retrieving connection to Redis at '{}'", self.addr);
        let timer = time::Instant::now();
        match self.pool.get().await {
            Ok(conn) => {
                info!(
                    "retrieved connection to Redis at '{}' in {:?}",
                    self.addr,
                    timer.elapsed()
                );
                Ok(conn)
            }
            Err(e) => Err(AppError::from(e)),
        }
    }
}
