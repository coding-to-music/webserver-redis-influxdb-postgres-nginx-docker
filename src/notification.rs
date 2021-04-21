use crate::{app::AppError, RedisPool};
use mobc_redis::{mobc::Connection, redis::AsyncCommands, RedisConnectionManager};
use std::sync::Arc;
use webserver_contracts::JsonRpcRequest;

pub struct NotificationHandler {
    pool: Arc<RedisPool>,
}

impl NotificationHandler {
    pub fn new(pool: Arc<RedisPool>) -> Self {
        Self { pool }
    }

    pub async fn publish_notification(&self, notification: JsonRpcRequest) -> Result<(), AppError> {
        let channel = notification_channel(&notification.method);
        let message = serde_json::to_string(&notification).unwrap();

        trace!("publishing notification on channel: '{}'", channel);

        let mut conn = self.get_connection().await?;

        conn.publish(channel, message).await?;

        Ok(())
    }

    async fn get_connection(&self) -> Result<Connection<RedisConnectionManager>, AppError> {
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => Err(AppError::from(e)),
        }
    }
}

fn notification_channel(method: &str) -> String {
    format!("ijagberg.notification.{}", method)
}
