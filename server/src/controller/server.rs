use crate::app::{AppResult, ParamsError};
use contracts::{server, JsonRpcRequest};
use server::sleep;
use std::{convert::TryFrom, time};

pub struct ServerController {}

impl ServerController {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn sleep(&self, request: JsonRpcRequest) -> AppResult<sleep::MethodResult> {
        use sleep::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let timer = time::Instant::now();
        tokio::time::sleep(time::Duration::from_millis(params.ms)).await;
        let elapsed = timer.elapsed();

        Ok(MethodResult::new(elapsed.as_millis() as u64))
    }
}

impl ParamsError for sleep::InvalidParams {}
