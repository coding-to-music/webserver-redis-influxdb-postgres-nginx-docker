use crate::app::{AppError, AppResult};
use jsonwebtoken::{
    errors::Error as JwtError, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use model::{JsonRpcError, Method};
use redis::{async_pool::mobc_redis::redis::AsyncCommands, async_pool::AsyncRedisPool};
use std::{collections::HashSet, fmt::Display, sync::Arc};

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
            let exp = chrono::Utc::now()
                .checked_add_signed(chrono::Duration::seconds(3600))
                .unwrap()
                .timestamp();
            let token = self.generate_token(exp);
            Ok(token)
        } else {
            Err(AppError::from(
                JsonRpcError::internal_error().with_message("invalid key name or key value"),
            ))
        }
    }

    pub fn parse_token(&self, token: &str) -> Result<Claims, JwtError> {
        let key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        match jsonwebtoken::decode(token, &key, &Validation::new(Algorithm::default())) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => {
                error!("failed to validate token with error: '{}'", e);
                Err(e)
            }
        }
    }

    fn generate_token(&self, expiry: i64) -> String {
        // let exp = chrono::Utc::now()
        //     .checked_add_signed(chrono::Duration::seconds(3600))
        //     .unwrap()
        //     .timestamp();
        jsonwebtoken::encode(
            &Header::default(),
            &Claims::new(expiry, vec![Role::User, Role::Anon]),
            &self.encoding_key,
        )
        .unwrap()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    exp: i64,
    roles: HashSet<String>,
}

impl Claims {
    pub(crate) fn new(exp: i64, roles: Vec<Role>) -> Self {
        Self {
            exp,
            roles: roles.into_iter().map(|r| r.to_string()).collect(),
        }
    }
}

pub fn authenticate(method: Method, claims: &Option<Claims>) -> Result<(), ()> {
    let roles = method_roles(method);
    match claims {
        Some(claims) => {
            if claims.roles.is_superset(&roles) {
                Ok(())
            } else {
                Err(())
            }
        }
        None => {
            if roles == vec![Role::Anon.to_string()].into_iter().collect() {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

pub fn method_roles(method: Method) -> HashSet<String> {
    use Role::*;
    match method {
        Method::GetDepartures => vec![Anon],
        _default => vec![SuperAdmin],
    }
    .into_iter()
    .map(|r| r.to_string())
    .collect()
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Role {
    SuperAdmin,
    Admin,
    User,
    Anon,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Role::SuperAdmin => "super_admin",
                Role::Admin => "admin",
                Role::User => "user",
                Role::Anon => "anon",
            }
        )
    }
}
