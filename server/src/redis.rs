use crate::app::AppResult;
use mobc_redis::{
    mobc::{Connection, Pool},
    redis::Client,
    RedisConnectionManager,
};
use std::time;

pub type RedisConnection = Connection<RedisConnectionManager>;

pub struct RedisPool {
    addr: String,
    pool: Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub fn new(addr: String) -> Self {
        let client = match Client::open(addr.clone()) {
            Ok(c) => c,
            Err(e) => panic!(
                "failed to create redis client with address: '{}', error: '{}'",
                addr, e
            ),
        };
        let pool = Pool::builder()
            .max_open(20)
            .build(RedisConnectionManager::new(client));
        Self { addr, pool }
    }

    pub async fn get_connection(&self) -> AppResult<RedisConnection> {
        trace!("retrieving connection to Redis at '{}'", self.addr);
        let timer = time::Instant::now();
        let conn = self.pool.get().await?;
        info!(
            "retrieved connection to Redis at '{}' in {:?}",
            self.addr,
            timer.elapsed()
        );
        Ok(conn)
    }
}
