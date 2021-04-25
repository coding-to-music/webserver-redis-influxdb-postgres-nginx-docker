use crate::{app::AppError, redis::RedisPool};
use contracts::{queue::QueueMessage, JsonRpcRequest};
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

    pub async fn publish_queue_message(&self, queue_message: QueueMessage) -> Result<(), AppError> {
        const QUEUE_CHANNEL: &str = "ijagberg.queue";
        let message = serde_json::to_string(&queue_message).unwrap();

        trace!("publishing queue message on channel: '{}'", QUEUE_CHANNEL);

        let mut conn = self.pool.get_connection().await?;
        conn.publish(QUEUE_CHANNEL, message).await?;

        Ok(())
    }
}

fn notification_channel(method: &str) -> String {
    format!("ijagberg.notification.{}", method)
}
