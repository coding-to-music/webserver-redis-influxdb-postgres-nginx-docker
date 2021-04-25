use super::Shape;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct Params {
    pub shapes: Vec<Shape>,
}

impl Params {
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self { shapes }
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    shapes: Vec<Shape>,
}

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Ok(Params::new(self.shapes))
    }
}

impl TryFrom<crate::JsonRpcRequest> for Params {
    type Error = InvalidParams;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
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
    pub ids: Vec<Option<String>>,
}

impl MethodResult {
    pub fn new(ids: Vec<Option<String>>) -> Self {
        Self { ids }
    }
}
