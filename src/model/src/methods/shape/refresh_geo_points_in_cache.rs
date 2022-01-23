use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub source: String,
}

impl Params {
    pub fn new(source: String) -> Result<Self, InvalidParams> {
        if source.is_empty() {
            return Err(InvalidParams::EmptySource);
        }
        Ok(Self { source })
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
        Params::new(builder.source)
    }
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    EmptySource,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            InvalidParams::EmptySource => crate::generic_invalid_value_message("source"),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    source: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    /// Number of geo points in cache after execution.
    pub count: usize,
}

impl MethodResult {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}
