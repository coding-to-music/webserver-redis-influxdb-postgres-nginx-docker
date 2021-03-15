use std::{convert::TryFrom, sync::Arc};

use webserver_contracts::{
    auth::{ValidateTokenParams, ValidateTokenParamsInvalid, ValidateTokenResult},
    Error as JsonRpcError, JsonRpcRequest,
};

use crate::{token::TokenHandler, AppError};

pub struct AuthController {
    token_handler: Arc<TokenHandler>,
}

impl AuthController {
    pub fn new(token_handler: Arc<TokenHandler>) -> Self {
        Self {
            token_handler,
        }
    }

    pub async fn validate_token(
        &self,
        request: JsonRpcRequest,
    ) -> Result<ValidateTokenResult, AppError> {
        let params = ValidateTokenParams::try_from(request)?;

        match self.token_handler.validate_token(&params.token) {
            Ok(_claims) => Ok(ValidateTokenResult::new(true)),
            Err(_e) => Ok(ValidateTokenResult::new(false)),
        }
    }
}

fn key(name: &str, value: &str) -> String {
    format!("{}-{}", name, value)
}

impl From<ValidateTokenParamsInvalid> for AppError {
    fn from(err: ValidateTokenParamsInvalid) -> Self {
        match err {
            ValidateTokenParamsInvalid::InvalidFormat(e) => JsonRpcError::invalid_format(e).into(),
        }
    }
}
