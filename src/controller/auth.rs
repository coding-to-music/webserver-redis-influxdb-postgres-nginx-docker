use std::convert::TryFrom;

use redis::Commands;
use webserver_contracts::{
    auth::{GetTokenParams, GetTokenParamsInvalid, GetTokenResult},
    Error as JsonRpcError, JsonRpcRequest,
};

use crate::AppError;

pub struct AuthController {
    redis_client: redis::Client,
}

impl AuthController {
    pub fn new(redis_addr: String) -> Self {
        let redis_client = redis::Client::open(redis_addr).unwrap();
        Self { redis_client }
    }

    pub async fn get_token(&self, request: JsonRpcRequest) -> Result<GetTokenResult, AppError> {
        let params = GetTokenParams::try_from(request)?;

        let mut conn = self.redis_client.get_connection()?;

        info!("connected to redis");
        let x: String = conn.get("test_key")?;

        info!("key has value: {}", x);
        Ok(GetTokenResult::new("asd".into()))
    }
}

impl From<GetTokenParamsInvalid> for AppError {
    fn from(err: GetTokenParamsInvalid) -> Self {
        match err {
            GetTokenParamsInvalid::InvalidFormat(e) => JsonRpcError::invalid_format(e).into(),
        }
    }
}
