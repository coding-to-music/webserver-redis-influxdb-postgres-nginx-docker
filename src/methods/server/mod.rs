pub use controller::ServerController;

mod controller;

use super::User;
use crate::Error;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize)]
pub struct SleepParamsBuilder {
    seconds: f32,
}

impl SleepParamsBuilder {
    pub fn build(self) -> Result<SleepParams, SleepParamsInvalid> {
        if self.seconds < 0.01 {
            Err(SleepParamsInvalid::SecondsTooLow)
        } else if self.seconds > 10.0 {
            Err(SleepParamsInvalid::SecondsTooHigh)
        } else {
            Ok(SleepParams {
                seconds: self.seconds,
            })
        }
    }
}

pub struct SleepParams {
    seconds: f32,
}

impl SleepParams {
    pub fn seconds(&self) -> f32 {
        self.seconds
    }
}

impl TryFrom<serde_json::Value> for SleepParams {
    type Error = SleepParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: SleepParamsBuilder =
            serde_json::from_value(value).map_err(SleepParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for SleepParams {
    type Error = SleepParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

impl From<SleepParamsInvalid> for Error {
    fn from(error: SleepParamsInvalid) -> Self {
        match error {
            SleepParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
            SleepParamsInvalid::SecondsTooLow => {
                Self::invalid_params().with_data("can't sleep for less than 0.01 seconds")
            }
            SleepParamsInvalid::SecondsTooHigh => {
                Self::invalid_params().with_data("can't sleep for more than 10.0 seconds")
            }
        }
    }
}

pub enum SleepParamsInvalid {
    InvalidFormat(serde_json::Error),
    SecondsTooLow,
    SecondsTooHigh,
}

#[derive(serde::Serialize)]
pub struct SleepResult {
    seconds: f32,
}

impl SleepResult {
    pub fn new(seconds: f32) -> Self {
        Self { seconds }
    }
}