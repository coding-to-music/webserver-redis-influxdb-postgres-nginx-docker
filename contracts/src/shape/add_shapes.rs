use super::Shape;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct AddShapesParams {
    pub shapes: Vec<Shape>,
}

impl AddShapesParams {
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self { shapes }
    }
}

#[derive(serde::Deserialize)]
struct AddShapesParamsBuilder {
    shapes: Vec<Shape>,
}

impl AddShapesParamsBuilder {
    fn build(self) -> Result<AddShapesParams, AddShapesParamsInvalid> {
        Ok(AddShapesParams::new(self.shapes))
    }
}

impl TryFrom<crate::JsonRpcRequest> for AddShapesParams {
    type Error = AddShapesParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: AddShapesParamsBuilder = serde_json::from_value(request.params)
            .map_err(AddShapesParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum AddShapesParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl Error for AddShapesParamsInvalid {}

impl Display for AddShapesParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            AddShapesParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AddShapesResult {
    pub ids: Vec<Option<String>>,
}

impl AddShapesResult {
    pub fn new(ids: Vec<Option<String>>) -> Self {
        Self { ids }
    }
}
