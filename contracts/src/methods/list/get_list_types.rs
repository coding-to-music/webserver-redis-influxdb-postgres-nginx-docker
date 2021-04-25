use crate::JsonRpcRequest;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Debug, Clone, serde::Serialize)]
#[non_exhaustive]
pub struct Params {}

impl Params {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {}

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Ok(Params::new())
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder = serde_json::from_value(request.params)
            .map_err(InvalidParams::InvalidFormat)?;

        builder.build()
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
    pub list_types: Vec<String>,
}

impl MethodResult {
    pub fn new(list_types: Vec<String>) -> Self {
        Self { list_types }
    }
}
