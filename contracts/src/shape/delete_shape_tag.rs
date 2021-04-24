use crate::JsonRpcRequest;
use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "DeleteShapeTagParamsBuilder")]
#[non_exhaustive]
pub struct DeleteShapeTagParams {
    pub id: Uuid,
}

impl DeleteShapeTagParams {
    pub fn new(id: Uuid) -> Result<Self, DeleteShapeTagParamsInvalid> {
        Ok(Self { id })
    }
}

#[derive(Debug, serde::Deserialize)]
struct DeleteShapeTagParamsBuilder {
    pub id: Uuid,
}

impl TryFrom<DeleteShapeTagParamsBuilder> for DeleteShapeTagParams {
    type Error = DeleteShapeTagParamsInvalid;

    fn try_from(value: DeleteShapeTagParamsBuilder) -> Result<Self, Self::Error> {
        Self::new(value.id)
    }
}

#[derive(Debug)]
pub enum DeleteShapeTagParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl Error for DeleteShapeTagParamsInvalid {}

impl Display for DeleteShapeTagParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            DeleteShapeTagParamsInvalid::InvalidFormat(e) => crate::invalid_params_serde_message(e),
        };

        write!(f, "{}", output)
    }
}

impl TryFrom<JsonRpcRequest> for DeleteShapeTagParams {
    type Error = DeleteShapeTagParamsInvalid;

    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        serde_json::from_value(request.params).map_err(Self::Error::InvalidFormat)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DeleteShapeTagResult {
    pub success: bool,
}

impl DeleteShapeTagResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
