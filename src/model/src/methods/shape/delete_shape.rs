use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub id: Uuid,
}

impl Params {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

impl TryFrom<ParamsBuilder> for Params {
    type Error = InvalidParams;

    fn try_from(value: ParamsBuilder) -> Result<Self, Self::Error> {
        Ok(Self::new(value.id))
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(value: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(value.params).map_err(InvalidParams::InvalidFormat)?;

        builder.try_into()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ParamsBuilder {
    id: Uuid,
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
