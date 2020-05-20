use super::User;
use std::convert::{TryFrom, TryInto};

pub use controller::MqttController;

mod controller;

pub struct PostLocalMqttParams {
    topic: String,
    payload: serde_json::Value,
    user: User,
}

#[derive(serde::Deserialize)]
struct PostLocalMqttParamsBuilder {
    topic: String,
    payload: serde_json::Value,
    user: User,
}

impl PostLocalMqttParamsBuilder {
    fn build(self) -> Result<PostLocalMqttParams, PostLocalMqttParamsInvalid> {
        if self.topic.is_empty() {
            Err(PostLocalMqttParamsInvalid::EmptyTopic)
        } else {
            Ok(PostLocalMqttParams {
                topic: self.topic,
                payload: self.payload,
                user: self.user,
            })
        }
    }
}

pub enum PostLocalMqttParamsInvalid {
    InvalidFormat(serde_json::Error),
    EmptyTopic,
}

impl TryFrom<serde_json::Value> for PostLocalMqttParams {
    type Error = PostLocalMqttParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: PostLocalMqttParamsBuilder =
            serde_json::from_value(value).map_err(PostLocalMqttParamsInvalid::InvalidFormat)?;
        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for PostLocalMqttParams {
    type Error = PostLocalMqttParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

impl From<PostLocalMqttParamsInvalid> for crate::Error {
    fn from(error: PostLocalMqttParamsInvalid) -> Self {
        match error {
            PostLocalMqttParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
            PostLocalMqttParamsInvalid::EmptyTopic => {
                Self::invalid_params().with_data("topic must not be empty")
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct PostLocalMqttResult {
    success: bool,
}
