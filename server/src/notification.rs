use crate::{app::AppError, redis::RedisPool};
use contracts::{queue::QueueMessage, JsonRpcRequest};
use mobc_redis::redis::AsyncCommands;
use std::sync::Arc;

pub struct NotificationHandler {
    config: Config,
    pool: Arc<RedisPool>,
}

impl NotificationHandler {
    pub fn new(config: Config, pool: Arc<RedisPool>) -> Self {
        Self { config, pool }
    }

    pub async fn publish_notification(&self, notification: JsonRpcRequest) -> Result<(), AppError> {
        let channel = self.notification_channel(&notification.method);
        let message = serde_json::to_string(&notification).unwrap();

        trace!("publishing notification on channel: '{}'", channel);

        let mut conn = self.pool.get_connection().await?;

        conn.publish(channel, message).await?;

        Ok(())
    }

    pub async fn publish_queue_message(&self, queue_message: QueueMessage) -> Result<(), AppError> {
        let queue_channel = &self.config.queue_channel;
        let message = serde_json::to_string(&queue_message).unwrap();

        trace!("publishing queue message on channel: '{}'", queue_channel);

        let mut conn = self.pool.get_connection().await?;
        conn.publish(queue_channel, message).await?;

        Ok(())
    }

    fn notification_channel(&self, method: &str) -> String {
        format!("{}.{}", self.config.notification_channel_prefix, method)
    }
}

pub struct Config {
    notification_channel_prefix: String,
    queue_channel: String,
}

impl Config {
    pub fn new(notification_channel_prefix: String, queue_channel: String) -> Self {
        Self {
            notification_channel_prefix,
            queue_channel,
        }
    }
}
