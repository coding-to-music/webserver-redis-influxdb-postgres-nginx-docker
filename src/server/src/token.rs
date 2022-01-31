use crate::app::{AppError, AppResult};
use jsonwebtoken::{
    errors::Error as JwtError, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use model::JsonRpcError;
use redis::{async_pool::mobc_redis::redis::AsyncCommands, async_pool::AsyncRedisPool};
use std::sync::Arc;

#[derive(Clone)]
pub struct TokenHandler {
    pool: Arc<AsyncRedisPool>,
    jwt_secret: String,
    encoding_key: EncodingKey,
}

impl TokenHandler {
    pub fn new(pool: Arc<AsyncRedisPool>, jwt_secret: String) -> Self {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        Self {
            pool,
            jwt_secret,
            encoding_key,
        }
    }

    pub async fn get_token(&self, key_name: &str, key_value: &str) -> AppResult<String> {
        let mut conn = self.pool.get_connection().await?;

        let redis_key = format!("{}-{}", key_name, key_value);

        trace!("retrieving key: '{}'", redis_key);

        let exists: bool = conn.exists(redis_key).await?;

        if exists {
            let token = self.generate_token();
            Ok(token)
        } else {
            Err(AppError::from(
                JsonRpcError::internal_error().with_message("invalid key name or key value"),
            ))
        }
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        match jsonwebtoken::decode(token, &key, &Validation::new(Algorithm::default())) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => {
                error!("failed to validate token with error: '{}'", e);
                Err(e)
            }
        }
    }

    fn generate_token(&self) -> String {
        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(3600))
            .unwrap()
            .timestamp();
        jsonwebtoken::encode(&Header::default(), &Claims::new(exp), &self.encoding_key).unwrap()
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
