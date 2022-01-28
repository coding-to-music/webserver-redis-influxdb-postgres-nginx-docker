use bb8_redis::{
    bb8::{self, PooledConnection},
    RedisConnectionManager as AsyncRedisConnectionManager,
};

pub struct AsyncRedisPool {
    pool: bb8::Pool<AsyncRedisConnectionManager>,
}

impl AsyncRedisPool {
    pub async fn new(addr: String) -> Self {
        let manager = AsyncRedisConnectionManager::new(addr.as_str()).unwrap();
        let pool = bb8::Pool::builder().build(manager).await.unwrap();
        Self { pool }
    }

    pub async fn get_connection(&self) -> PooledConnection<'_, AsyncRedisConnectionManager> {
        let conn = self.pool.get().await.unwrap();
        conn
    }
}

pub struct SyncRedisPool {
    pool: r2d2::Pool<redis::Client>,
}

impl SyncRedisPool {
    pub fn new(addr: String) -> Self {
        let client: redis::Client = redis::Client::open(addr)
            .unwrap_or_else(|e| panic!("Error connecting to redis: {}", e));
        // create r2d2 pool
        let pool: r2d2::Pool<redis::Client> = r2d2::Pool::builder()
            .max_size(15)
            .build(client)
            .unwrap_or_else(|e| panic!("Error building redis pool: {}", e));
        Self { pool }
    }

    pub fn get_connection(&self) -> r2d2::PooledConnection<redis::Client> {
        self.pool.get().unwrap()
    }
}
