use std::{convert::TryFrom, time};

use crate::{token::TokenHandler, AppError};
use webserver_contracts::{
    server::{SleepParams, SleepParamsInvalid, SleepResult},
    Error as JsonRpcError, JsonRpcRequest,
};

pub struct ServerController {
    token_handler: TokenHandler,
}

impl ServerController {
    pub fn new(token_handler: TokenHandler) -> Self {
        Self { token_handler }
    }

    pub async fn sleep(&self, request: JsonRpcRequest) -> Result<SleepResult, AppError> {
        let params = SleepParams::try_from(request)?;
        self.token_handler.validate_token(&params.token)?;

        let timer = time::Instant::now();
        tokio::time::delay_for(time::Duration::from_millis(params.ms)).await;
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
