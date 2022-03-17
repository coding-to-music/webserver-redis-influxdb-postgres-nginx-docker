use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};

use super::User;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub id: String,
}

impl Params {
    pub fn new(id: String) -> Result<Self, InvalidParams> {
        Ok(Self { id })
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(request.params).map_err(InvalidParams::InvalidFormat)?;
        builder.try_into()
    }
}

impl TryFrom<ParamsBuilder> for Params {
    type Error = InvalidParams;

    fn try_from(builder: ParamsBuilder) -> Result<Self, Self::Error> {
        Params::new(builder.id)
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    id: String,
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };
        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub user: Option<User>,
}

impl MethodResult {
    fn new(user: Option<User>) -> Self {
        Self { user }
    }

    pub fn exists(user: User) -> Self {
        Self::new(Some(user))
    }

    pub fn missing() -> Self {
        Self::new(None)
    }
}
