use super::*;
use crate::db;
use rumq_client::{MqttEventLoop, MqttOptions, Publish, QoS, Request};
use std::{convert::TryInto, sync::Arc};
use tokio::sync::{mpsc, Mutex};

pub struct MqttController {
    user_db: Arc<db::Database<db::User>>,
    sender: Mutex<mpsc::Sender<rumq_client::Request>>,
    eventloop: MqttEventLoop,
}

impl MqttController {
    pub fn new(user_db: Arc<db::Database<db::User>>) -> Self {
        let (req_tx, req_rx) = mpsc::channel(100);
        let mut opts = MqttOptions::new("webserver", "localhost", 1883);
        opts.set_keep_alive(5)
            .set_throttle(std::time::Duration::from_secs(1))
            .set_clean_session(true)
            .set_max_packet_size(100_000);
        let eventloop = rumq_client::eventloop(opts, req_rx);
        Self {
            sender: Mutex::new(req_tx),
            eventloop,
            user_db,
        }
    }

    pub async fn post_local<T>(&self, params: T) -> Result<PostLocalMqttResult, crate::Error>
    where
        T: TryInto<PostLocalMqttParams, Error = PostLocalMqttParamsInvalid>,
    {
        let params: PostLocalMqttParams = params.try_into()?;

        if !self.user_db.validate_user(&params.user) {
            return Err(crate::Error::invalid_params().with_data("invalid username or password"));
        }

        info!("publishing message to mqtt topic: '{}'", params.topic);

        let request = Publish::new(
            params.topic,
            QoS::AtLeastOnce,
            serde_json::to_vec(&params.payload)
                .map_err(|e| crate::Error::internal_error().with_internal_data(e))?,
        );

        self.sender
            .lock()
            .await
            .send(Request::Publish(request))
            .await
            .map_err(|e| crate::Error::internal_error().with_internal_data(e))?;

        Ok(PostLocalMqttResult { success: true })
    }
}
