use crate::app::{AppError, AppResult};
use mobc_redis::{
    mobc::{Connection, Pool},
    redis::Client,
    RedisConnectionManager,
};

pub struct RedisPool {
    pool: Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub fn new(addr: &str) -> Self {
        let pool = Pool::builder()
            .max_open(20)
            .build(RedisConnectionManager::new(Client::open(addr).unwrap()));
        Self { pool }
    }

    pub async fn get_connection(&self) -> AppResult<Connection<RedisConnectionManager>> {
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => Err(AppError::from(e)),
        }
    }
}
