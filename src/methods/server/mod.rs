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

impl TryFrom<crate::JsonRpcRequest> for SleepParams {
    type Error = SleepParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: SleepParamsBuilder =
            serde_json::from_value(request.params).map_err(SleepParamsInvalid::InvalidFormat)?;

        builder.build()
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
    dry_run: bool,
}

#[derive(serde::Deserialize)]
struct ClearLogsParamsBuilder {
    user: User,
    dry_run: bool,
}

impl ClearLogsParamsBuilder {
    fn build(self) -> Result<ClearLogsParams, ClearLogsParamsInvalid> {
        Ok(ClearLogsParams {
            user: self.user,
            dry_run: self.dry_run,
        })
    }
}

impl TryFrom<crate::JsonRpcRequest> for ClearLogsParams {
    type Error = ClearLogsParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ClearLogsParamsBuilder = serde_json::from_value(request.params)
            .map_err(ClearLogsParamsInvalid::InvalidFormat)?;

        builder.build()
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
    dry_run: bool,
    files: usize,
    bytes: u64,
}
