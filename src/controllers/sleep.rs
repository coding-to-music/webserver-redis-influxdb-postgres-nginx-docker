use crate::app::Error;
use core::convert::{TryFrom, TryInto};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub struct SleepController;

impl SleepController {
    pub fn new() -> Self {
        Self
    }

    pub(crate) async fn sleep<T: TryInto<SleepParams, Error = Error>>(
        &self,
        params: T,
    ) -> Result<SleepResult, Error> {
        let params = params.try_into()?;

        info!("sleeping for {}s", params.seconds);
        let start = std::time::Instant::now();
        tokio::time::delay_for(std::time::Duration::from_secs(params.seconds)).await;
        let slept_s = start.elapsed().as_secs();
        Ok(SleepResult { slept_s })
    }
}

#[derive(Deserialize)]
pub struct SleepParams {
    seconds: u64,
}

#[derive(Serialize)]
pub struct SleepResult {
    slept_s: u64,
}

impl TryFrom<Value> for SleepParams {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let params = serde_json::from_value::<Self>(value).map_err(|_| Error::invalid_params())?;

        if params.seconds > 10 {
            Err(Error::invalid_params().with_message("seconds cannot be higher than 10"))
        } else {
            Ok(params)
        }
    }
}
