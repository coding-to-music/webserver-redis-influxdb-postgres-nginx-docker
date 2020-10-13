use std::{convert::TryFrom, sync::Arc};

use influx::{InfluxClient, Measurement};
use webserver_contracts::mqtt::{
    HandleMqttMessageParams, HandleMqttMessageParamsInvalid, HandleMqttMessageResult, MqttMessage,
};
use webserver_contracts::Error as JsonRpcError;

use crate::AppError;

pub struct MqttController {
    influx_client: Arc<InfluxClient>,
}

impl MqttController {
    pub fn new(influx_client: Arc<InfluxClient>) -> Self {
        Self { influx_client }
    }

    pub async fn handle_mqtt_message(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<HandleMqttMessageResult, AppError> {
        let params = HandleMqttMessageParams::try_from(request)?;

        let wrapper = MqttMessageWrapper(params.message());

        let influx_response = self
            .influx_client
            .send_batch("mqtt".into(), &[Measurement::from(wrapper)])
            .await;

        Ok(HandleMqttMessageResult::new(
            influx_response.status().is_success(),
        ))
    }
}

struct MqttMessageWrapper<'a>(&'a MqttMessage);

impl From<MqttMessageWrapper<'_>> for Measurement {
    fn from(w: MqttMessageWrapper) -> Self {
        let measurement = Measurement::builder("mqtt_message".into())
            .with_tag("topic".into(), w.0.topic().into())
            .with_field_string("payload".into(), w.0.payload().into())
            .build()
            .unwrap();

        measurement
    }
}

impl From<HandleMqttMessageParamsInvalid> for AppError {
    fn from(error: HandleMqttMessageParamsInvalid) -> Self {
        match error {
            HandleMqttMessageParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}
