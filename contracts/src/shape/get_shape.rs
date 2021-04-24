use super::Shape;
use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct GetShapeParams {
    pub id: Uuid,
}

impl GetShapeParams {
    pub fn new(id: Uuid) -> Result<Self, GetShapeParamsInvalid> {
        Ok(Self { id })
    }
}

#[derive(Debug)]
pub enum GetShapeParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl Error for GetShapeParamsInvalid {}

impl Display for GetShapeParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            GetShapeParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Deserialize)]
struct GetShapeParamsBuilder {
    id: Uuid,
}

impl GetShapeParamsBuilder {
    fn build(self) -> Result<GetShapeParams, GetShapeParamsInvalid> {
        GetShapeParams::new(self.id)
    }
}

impl TryFrom<crate::JsonRpcRequest> for GetShapeParams {
    type Error = GetShapeParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: GetShapeParamsBuilder =
            serde_json::from_value(request.params).map_err(GetShapeParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GetShapeResult {
    pub shape: Option<Shape>,
}

impl GetShapeResult {
    pub fn new(shape: Option<Shape>) -> Self {
        Self { shape }
    }
}
