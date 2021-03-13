use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use webserver_contracts::Error as JsonRpcError;

use crate::AppError;

#[derive(Debug, Clone)]
pub struct TokenHandler {
    jwt_secret: String,
}

impl TokenHandler {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        match jsonwebtoken::decode(token, &key, &Validation::new(Algorithm::default())) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => Err(AppError::from(JsonRpcError::not_permitted()).with_context(&e)),
        }
    }

    pub fn generate_token(&self) -> String {
        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(3600))
            .unwrap()
            .timestamp();
        jsonwebtoken::encode(
            &Header::default(),
            &Claims::new(exp),
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .unwrap()
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    exp: i64,
}

impl Claims {
    pub fn new(exp: i64) -> Self {
        Self { exp }
    }
}
