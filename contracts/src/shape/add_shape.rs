use super::Shape;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct AddShapeParams {
    pub shape: Shape,
}

impl AddShapeParams {
    pub fn new(shape: Shape) -> Self {
        Self { shape }
    }
}

#[derive(serde::Deserialize)]
struct AddShapeParamsBuilder {
    shape: Shape,
}

impl AddShapeParamsBuilder {
    fn build(self) -> Result<AddShapeParams, AddShapeParamsInvalid> {
        Ok(AddShapeParams::new(self.shape))
    }
}

impl TryFrom<crate::JsonRpcRequest> for AddShapeParams {
    type Error = AddShapeParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: AddShapeParamsBuilder =
            serde_json::from_value(request.params).map_err(AddShapeParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum AddShapeParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl Error for AddShapeParamsInvalid {}

impl Display for AddShapeParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            AddShapeParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AddShapeResult {
    pub success: bool,
    pub id: Option<String>,
}

impl AddShapeResult {
    pub fn success(id: String) -> Self {
        Self {
            success: true,
            id: Some(id),
        }
    }

    pub fn failure() -> Self {
        Self {
            success: false,
            id: None,
        }
    }
}
