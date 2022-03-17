use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};

use super::Departure;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
   pub stop_id: String,
   pub count: u32,
}

impl Params {
    pub fn new(stop_id: String, count: u32) -> Self {
        Self { stop_id, count }
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
        Ok(Params::new(builder.stop_id, builder.count))
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    stop_id: String,
    count: u32,
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
    pub departures: Vec<Departure>,
}

impl MethodResult {
    pub fn new(departures: Vec<Departure>) -> Self {
        Self { departures }
    }
}
