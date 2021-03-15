use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use redis::Commands;

#[derive(Debug, Clone)]
pub struct TokenHandler {
    redis_addr: String,
    jwt_secret: String,
    encoding_key: EncodingKey,
}

impl TokenHandler {
    pub fn new(redis_addr: String, jwt_secret: String) -> Self {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        Self {
            redis_addr,
            jwt_secret,
            encoding_key,
        }
    }

    pub fn get_token(&self, key_name: &str, key_value: &str) -> Result<String, ()> {
        let mut redis_client = redis::Client::open(self.redis_addr.clone()).map_err(|_| ())?;

        let exists: bool = redis_client
            .exists(format!("{}-{}", key_name, key_value))
            .map_err(|_| ())?;

        if exists {
            let token = self.generate_token();
            Ok(token)
        } else {
            Err(())
        }
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, ()> {
        let key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        match jsonwebtoken::decode(token, &key, &Validation::new(Algorithm::default())) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => {
                error!("failed to validate token with error: '{}'", e);
                Err(())
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
