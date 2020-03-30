use std::convert::{TryFrom, TryInto};

pub struct SleepController {}

impl SleepController {
    pub fn new() -> Self {
        info!("Creating new SleepController");
        Self {}
    }

    pub async fn sleep<T: TryInto<SleepParams, Error = SleepParamsInvalid>>(
        &self,
        p: T,
    ) -> Result<SleepResult, super::Error> {
        let params: SleepParams = p.try_into()?;
        tokio::time::delay_for(std::time::Duration::from_secs(params.seconds.into())).await;

        Ok(SleepResult {})
    }
}

#[derive(serde::Deserialize)]
pub struct SleepParams {
    seconds: u32,
}

impl TryFrom<serde_json::Value> for SleepParams {
    type Error = SleepParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let params: SleepParams =
            serde_json::from_value(value).map_err(|_| SleepParamsInvalid::InvalidFormat)?;

        if params.seconds > 10 {
            Err(Self::Error::SecondsTooHigh)
        } else if params.seconds == 0 {
            Err(Self::Error::SecondsAreZero)
        } else {
            Ok(params)
        }
    }
}

#[derive(Debug)]
pub enum SleepParamsInvalid {
    SecondsTooHigh,
    SecondsAreZero,
    InvalidFormat,
}

impl From<SleepParamsInvalid> for super::Error {
    fn from(_: SleepParamsInvalid) -> Self {
        Self::invalid_params()
    }
}

#[derive(serde::Serialize)]
pub struct SleepResult {}
