use crate::JsonRpcRequest;
use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub id: Uuid,
}

impl Params {
    pub fn new(id: Uuid) -> Result<Self, InvalidParams> {
        Ok(Self { id })
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ParamsBuilder {
    id: Uuid,
}

impl TryFrom<ParamsBuilder> for Params {
    type Error = InvalidParams;

    fn try_from(value: ParamsBuilder) -> Result<Self, Self::Error> {
        Self::new(value.id)
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(value: JsonRpcRequest) -> Result<Self, Self::Error> {
        serde_json::from_value(value.params).map_err(InvalidParams::InvalidFormat)
    }
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(e) => crate::invalid_params_serde_message(e),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub success: bool,
}

impl MethodResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
