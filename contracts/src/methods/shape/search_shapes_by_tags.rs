use super::Shape;
use crate::JsonRpcRequest;
use std::{collections::HashMap, convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub or: Vec<HashMap<String, String>>,
}

impl Params {
    pub fn new(or: Vec<HashMap<String, String>>) -> Result<Self, InvalidParams> {
        let o = Self { or };

        o.validate()?;

        Ok(o)
    }

    fn validate(&self) -> Result<(), InvalidParams> {
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
struct ParamsBuilder {
    or: Vec<HashMap<String, String>>,
}

impl TryFrom<ParamsBuilder> for Params {
    type Error = InvalidParams;

    fn try_from(builder: ParamsBuilder) -> Result<Self, Self::Error> {
        Self::new(builder.or)
    }
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    InvalidName,
    InvalidValue,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(e) => {
                crate::invalid_params_serde_message(e)
            }
            InvalidParams::InvalidName => "invalid tag name".to_string(),
            InvalidParams::InvalidValue => "invalid tag value".to_string(),
        };

        write!(f, "{}", output)
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        serde_json::from_value(request.params).map_err(Self::Error::InvalidFormat)
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
