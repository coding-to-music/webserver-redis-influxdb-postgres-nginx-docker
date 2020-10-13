pub use handle_mqtt_message::{
    HandleMqttMessageParams, HandleMqttMessageParamsInvalid, HandleMqttMessageResult,
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MqttMessage {
    topic: String,
    payload: String,
}

impl MqttMessage {
    pub fn new(topic: String, payload: String) -> Self {
        Self { topic, payload }
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }
}

mod handle_mqtt_message {
    use super::*;
    use std::convert::TryFrom;

    pub struct HandleMqttMessageParams {
        message: MqttMessage,
    }

    impl HandleMqttMessageParams {
        fn new(message: MqttMessage) -> Self {
            Self { message }
        }

        pub fn message(&self) -> &MqttMessage {
            &self.message
        }
    }

    #[derive(serde::Deserialize)]
    struct HandleMqttMessageParamsBuilder {
        message: MqttMessage,
    }

    impl HandleMqttMessageParamsBuilder {
        fn build(self) -> Result<HandleMqttMessageParams, HandleMqttMessageParamsInvalid> {
            Ok(HandleMqttMessageParams::new(self.message))
        }
    }

    #[derive(Debug)]
    pub enum HandleMqttMessageParamsInvalid {
        InvalidFormat(serde_json::Error),
    }

    impl TryFrom<crate::JsonRpcRequest> for HandleMqttMessageParams {
        type Error = HandleMqttMessageParamsInvalid;

        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: HandleMqttMessageParamsBuilder = serde_json::from_value(request.params)
                .map_err(HandleMqttMessageParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    #[derive(Debug, serde::Serialize)]
    pub struct HandleMqttMessageResult {
        success: bool,
    }

    impl HandleMqttMessageResult {
        pub fn new(success: bool) -> Self {
            Self { success }
        }
    }
}
