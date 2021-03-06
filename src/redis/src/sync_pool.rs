pub use r2d2_redis;

use r2d2_redis::{
    r2d2::{self, PooledConnection},
    RedisConnectionManager,
};
use std::{error::Error, time};

pub struct SyncRedisPool {
    addr: String,
    pool: r2d2_redis::r2d2::Pool<RedisConnectionManager>,
}

impl SyncRedisPool {
    pub fn new(addr: String, count: u32) -> Self {
        let manager = RedisConnectionManager::new(addr.clone()).unwrap();
        let pool = r2d2::Pool::builder()
            .max_size(count)
            .build(manager)
            .unwrap();
        Self { addr, pool }
    }

    pub fn get_connection(
        &self,
    ) -> Result<PooledConnection<RedisConnectionManager>, Box<dyn Error>> {
        trace!("retrieving connection to Redis at '{}'", self.addr);
        let timer = time::Instant::now();
        let conn = self.pool.get()?;
        info!(
            "retrieved connection to Redis at '{}' in {:?}",
            self.addr,
            timer.elapsed()
        );
        Ok(conn)
    }
}

// let manager = RedisConnectionManager::new("redis://localhost").unwrap();
//     let pool = r2d2::Pool::builder()
//         .build(manager)
//         .unwrap();
