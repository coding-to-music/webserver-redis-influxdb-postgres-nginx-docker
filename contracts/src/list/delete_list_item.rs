use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
#[non_exhaustive]
pub struct DeleteListItemParams {
    pub id: Uuid,
}

impl DeleteListItemParams {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

#[derive(serde::Deserialize)]
struct DeleteListItemParamsBuilder {
    id: Uuid,
}

impl DeleteListItemParamsBuilder {
    fn build(self) -> Result<DeleteListItemParams, DeleteListItemParamsInvalid> {
        Ok(DeleteListItemParams::new(self.id))
    }
}

impl TryFrom<crate::JsonRpcRequest> for DeleteListItemParams {
    type Error = DeleteListItemParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: DeleteListItemParamsBuilder = serde_json::from_value(request.params)
            .map_err(DeleteListItemParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum DeleteListItemParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl Error for DeleteListItemParamsInvalid {}

impl Display for DeleteListItemParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            DeleteListItemParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DeleteListItemResult {
    pub deleted: bool,
}

impl DeleteListItemResult {
    pub fn new(deleted: bool) -> Self {
        Self { deleted }
    }
}
