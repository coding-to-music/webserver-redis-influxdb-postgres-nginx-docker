use crate::{app::AppError, redis::RedisPool};
use contracts::JsonRpcRequest;
use mobc_redis::redis::AsyncCommands;
use std::sync::Arc;

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

        let mut conn = self.pool.get_connection().await?;

        conn.publish(channel, message).await?;

        Ok(())
    }
}

fn notification_channel(method: &str) -> String {
    format!("ijagberg.notification.{}", method)
}
