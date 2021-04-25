use crate::JsonRpcRequest;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct Params {
    pub ms: u64,
}

impl Params {
    pub fn new(ms: u64) -> Result<Self, InvalidParams> {
        if ms > 10_000 {
            Err(InvalidParams::InvalidDuration)
        } else {
            Ok(Self { ms })
        }
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    pub ms: u64,
}

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Params::new(self.ms)
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(value: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(value.params).map_err(Self::Error::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    InvalidDuration,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            InvalidParams::InvalidDuration => "'duration' has an invalid value".to_string(),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub slept_ms: u64,
}

impl MethodResult {
    pub fn new(slept_ms: u64) -> Self {
        Self { slept_ms }
    }
}