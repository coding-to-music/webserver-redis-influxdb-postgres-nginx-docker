use super::Shape;
use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct Params {
    pub id: Uuid,
}

impl Params {
    pub fn new(id: Uuid) -> Result<Self, InvalidParams> {
        Ok(Self { id })
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

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    id: Uuid,
}

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Params::new(self.id)
    }
}

impl TryFrom<crate::JsonRpcRequest> for Params {
    type Error = InvalidParams;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(request.params).map_err(InvalidParams::InvalidFormat)?;

        builder.build()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub shape: Option<Shape>,
}

impl MethodResult {
    pub fn new(shape: Option<Shape>) -> Self {
        Self { shape }
    }
}