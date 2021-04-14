use crate::{app::ParamsError, AppError};
use std::{convert::TryFrom, time};
use webserver_contracts::{
    server::{SleepParams, SleepParamsInvalid, SleepResult},
    JsonRpcRequest,
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

impl ParamsError for SleepParamsInvalid {}
