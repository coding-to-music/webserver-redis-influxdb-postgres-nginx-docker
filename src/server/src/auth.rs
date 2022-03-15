use jsonwebtoken::{
    errors::Error as JwtError, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use model::Method;
use std::{collections::HashSet, fmt::Display};

#[derive(Clone)]
pub struct TokenHandler {
    jwt_secret: String,
    encoding_key: EncodingKey,
}

impl TokenHandler {
    pub fn new(jwt_secret: String) -> Self {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        Self {
            jwt_secret,
            encoding_key,
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

    pub(crate) fn generate_token(&self, expiry: i64, mut roles: Vec<Role>) -> Option<String> {
        roles.push(Role::User);
        roles.push(Role::Anon);
        jsonwebtoken::encode(
            &Header::default(),
            &Claims::new(expiry, roles),
            &self.encoding_key,
        )
        .ok()
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
        Method::GetToken => vec![Anon],
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

impl Role {
    pub(crate) fn from_sql_value(sql_value: &str) -> Result<Role, ()> {
        match sql_value {
            "SuperAdmin" => Ok(Role::SuperAdmin),
            "Admin" => Ok(Role::Admin),
            "User" => Ok(Role::User),
            "Anon" => Ok(Role::Anon),
            invalid => {
                error!("failed to parse '{}' as a Role", invalid);
                Err(())
            }
        }
    }
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
