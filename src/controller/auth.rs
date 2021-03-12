use std::convert::TryFrom;

use jsonwebtoken::{EncodingKey, Header};
use redis::Commands;
use webserver_contracts::{
    auth::{
        Claims, GetTokenParams, GetTokenParamsInvalid, GetTokenResult, ValidateTokenParams,
        ValidateTokenParamsInvalid, ValidateTokenResult,
    },
    Error as JsonRpcError, JsonRpcRequest,
};

use crate::AppError;

pub struct AuthController {
    redis_client: redis::Client,
    jwt_secret: String,
}

impl AuthController {
    pub fn new(redis_addr: String, jwt_secret: String) -> Self {
        let redis_client = redis::Client::open(redis_addr).unwrap();
        Self {
            redis_client,
            jwt_secret,
        }
    }

    pub async fn get_token(&self, request: JsonRpcRequest) -> Result<GetTokenResult, AppError> {
        let params = GetTokenParams::try_from(request)?;

        let mut conn = self.redis_client.get_connection()?;

        let key = key(&params.key_name, &params.key);

        if conn.exists(key)? {
            info!("key exists");
            let token = self.generate_token(&params.claims);

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

        match crate::validate_token(&params.token, &self.jwt_secret) {
            Ok(_claims) => Ok(ValidateTokenResult::new(true)),
            Err(_) => Ok(ValidateTokenResult::new(false)),
        }
    }

    fn generate_token(&self, claims: &Claims) -> String {
        jsonwebtoken::encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_test() {
        let controller = AuthController::new("redis://localhost".to_owned(), "test".to_owned());
        let token = controller.generate_token(&Claims::new("admin".to_owned()));

        println!("Token: '{}'", token);

        assert_eq!(token.as_str(), "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoiYWRtaW4ifQ.oNJmSCDLzVHLqGVM6OVtQdSrhoUiRI3VlZkCe4f0NxY");
    }
}
