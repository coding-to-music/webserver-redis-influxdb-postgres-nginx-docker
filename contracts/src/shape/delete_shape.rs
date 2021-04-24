use std::{convert::TryFrom, error::Error, fmt::Display};

use uuid::Uuid;

use crate::JsonRpcRequest;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "DeleteShapeParamsBuilder")]
#[non_exhaustive]
pub struct DeleteShapeParams {
    pub id: Uuid,
}

impl DeleteShapeParams {
    pub fn new(id: Uuid) -> Result<Self, DeleteShapeParamsInvalid> {
        Ok(Self { id })
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct DeleteShapeParamsBuilder {
    id: Uuid,
}

impl TryFrom<DeleteShapeParamsBuilder> for DeleteShapeParams {
    type Error = DeleteShapeParamsInvalid;

    fn try_from(value: DeleteShapeParamsBuilder) -> Result<Self, Self::Error> {
        Self::new(value.id)
    }
}

impl TryFrom<JsonRpcRequest> for DeleteShapeParams {
    type Error = DeleteShapeParamsInvalid;

    fn try_from(value: JsonRpcRequest) -> Result<Self, Self::Error> {
        serde_json::from_value(value.params).map_err(DeleteShapeParamsInvalid::InvalidFormat)
    }
}

#[derive(Debug)]
pub enum DeleteShapeParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl Error for DeleteShapeParamsInvalid {}

impl Display for DeleteShapeParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            DeleteShapeParamsInvalid::InvalidFormat(e) => crate::invalid_params_serde_message(e),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DeleteShapeResult {
    pub success: bool,
}

impl DeleteShapeResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
