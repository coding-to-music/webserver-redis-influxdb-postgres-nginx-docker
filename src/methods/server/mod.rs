pub use controller::ServerController;

mod controller;

use super::User;
use crate::Error;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize)]
pub struct SleepParamsBuilder {
    seconds: f32,
    user: User,
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
                user: self.user,
            })
        }
    }
}

pub struct SleepParams {
    user: User,
    seconds: f32,
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

pub struct ClearLogsParams {
    user: User,
}

#[derive(serde::Deserialize)]
struct ClearLogsParamsBuilder {
    user: User,
}

impl ClearLogsParamsBuilder {
    fn build(self) -> Result<ClearLogsParams, ClearLogsParamsInvalid> {
        Ok(ClearLogsParams { user: self.user })
    }
}

impl TryFrom<serde_json::Value> for ClearLogsParams {
    type Error = ClearLogsParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: ClearLogsParamsBuilder =
            serde_json::from_value(value).map_err(ClearLogsParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for ClearLogsParams {
    type Error = ClearLogsParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

impl From<ClearLogsParamsInvalid> for crate::Error {
    fn from(error: ClearLogsParamsInvalid) -> Self {
        match error {
            ClearLogsParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
        }
    }
}

pub enum ClearLogsParamsInvalid {
    InvalidFormat(serde_json::Error),
}

#[derive(serde::Serialize)]
pub struct ClearLogsResult {
    files: usize,
    bytes: u64,
}
