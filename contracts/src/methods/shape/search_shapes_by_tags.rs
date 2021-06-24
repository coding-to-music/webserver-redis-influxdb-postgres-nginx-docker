use super::Shape;
use crate::JsonRpcRequest;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub or: Vec<HashMap<String, String>>,
}

impl Params {
    pub fn new(or: Vec<HashMap<String, String>>) -> Self {
        Self { or }
    }
}

impl TryFrom<ParamsBuilder> for Params {
    type Error = InvalidParams;

    fn try_from(builder: ParamsBuilder) -> Result<Self, Self::Error> {
        Ok(Self::new(builder.or))
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(request.params).map_err(Self::Error::InvalidFormat)?;

        builder.try_into()
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    or: Vec<HashMap<String, String>>,
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
    pub shapes: Vec<Shape>,
}

impl MethodResult {
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self { shapes }
    }
}
