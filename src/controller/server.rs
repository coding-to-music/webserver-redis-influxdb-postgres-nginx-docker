use std::{convert::TryFrom, time};

use crate::AppError;
use webserver_contracts::{
    server::{SleepParams, SleepParamsInvalid, SleepResult},
    JsonRpcError, JsonRpcRequest,
};

pub struct ServerController {}

impl ServerController {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn sleep(&self, request: JsonRpcRequest) -> Result<SleepResult, AppError> {
        let params = SleepParams::try_from(request)?;

        let timer = time::Instant::now();
        tokio::time::sleep(time::Duration::from_millis(params.ms)).await;
        let elapsed = timer.elapsed();

        Ok(SleepResult::new(elapsed.as_millis() as u64))
    }
}

impl From<SleepParamsInvalid> for AppError {
    fn from(error: SleepParamsInvalid) -> Self {
        match error {
            SleepParamsInvalid::InvalidFormat(e) => JsonRpcError::invalid_format(e).into(),
            SleepParamsInvalid::InvalidDuration => JsonRpcError::invalid_params()
                .with_message("invalid duration, should be less than or equal to 10000")
                .into(),
        }
    }
}
