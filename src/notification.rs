use redis::AsyncCommands;
use webserver_contracts::JsonRpcRequest;

use crate::app::AppError;

pub struct NotificationHandler {
    redis: redis::Client,
}

impl NotificationHandler {
    pub fn new(redis_addr: String) -> Self {
        let redis = redis::Client::open(redis_addr).unwrap();
        Self { redis }
    }

    pub async fn publish_notification(&self, notification: JsonRpcRequest) -> Result<(), AppError> {
        let channel = notification_channel(&notification.method);
        let message = serde_json::to_string(&notification).unwrap();

        trace!("publishing notification on channel: '{}'", channel);

        let mut conn = self.redis.get_async_connection().await?;

        conn.publish(channel, message).await?;

        Ok(())
    }
}

fn notification_channel(method: &str) -> String {
    format!("ijagberg.notification.{}", method)
}
