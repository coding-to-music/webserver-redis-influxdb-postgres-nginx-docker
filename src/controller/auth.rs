use std::convert::TryFrom;

use redis::Commands;
use webserver_contracts::{
    auth::{
        GetTokenParams, GetTokenParamsInvalid, GetTokenResult, ValidateTokenParams,
        ValidateTokenParamsInvalid, ValidateTokenResult,
    },
    Error as JsonRpcError, JsonRpcRequest,
};

use crate::{token::TokenHandler, AppError};

pub struct AuthController {
    redis_client: redis::Client,
    token_handler: TokenHandler,
}

impl AuthController {
    pub fn new(redis_addr: String, token_handler: TokenHandler) -> Self {
        let redis_client = redis::Client::open(redis_addr).unwrap();
        Self {
            redis_client,
            token_handler,
        }
    }

    pub async fn get_token(&self, request: JsonRpcRequest) -> Result<GetTokenResult, AppError> {
        let params = GetTokenParams::try_from(request)?;

        let mut conn = self.redis_client.get_connection()?;

        let key = key(&params.key_name, &params.key);

        if conn.exists(key)? {
            info!("key exists");
            let token = self.token_handler.generate_token();

            Ok(GetTokenResult::new(token))
        } else {
            warn!("key does not exist");
            Err(JsonRpcError::not_permitted().into())
        }
    }

    pub async fn validate_token(
        &self,
        request: JsonRpcRequest,
    ) -> Result<ValidateTokenResult, AppError> {
        let params = ValidateTokenParams::try_from(request)?;

        match self.token_handler.validate_token(&params.token) {
            Ok(_claims) => Ok(ValidateTokenResult::new(true)),
            Err(_e) => {
                Ok(ValidateTokenResult::new(false))
            }
        }
    }
}

fn key(name: &str, value: &str) -> String {
    format!("{}-{}", name, value)
}

impl From<GetTokenParamsInvalid> for AppError {
    fn from(err: GetTokenParamsInvalid) -> Self {
        match err {
            GetTokenParamsInvalid::InvalidFormat(e) => JsonRpcError::invalid_format(e).into(),
            GetTokenParamsInvalid::InvalidClaims => JsonRpcError::invalid_request()
                .with_message("invalid claims")
                .into(),
        }
    }
}

impl From<ValidateTokenParamsInvalid> for AppError {
    fn from(err: ValidateTokenParamsInvalid) -> Self {
        match err {
            ValidateTokenParamsInvalid::InvalidFormat(e) => JsonRpcError::invalid_format(e).into(),
        }
    }
}
