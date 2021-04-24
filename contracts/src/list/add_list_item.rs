use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct AddListItemParams {
    pub id: Option<Uuid>,
    pub list_type: String,
    pub item_name: String,
}

impl AddListItemParams {
    pub fn new(id: Option<Uuid>, list_type: String, item_name: String) -> Self {
        Self {
            id,
            list_type,
            item_name,
        }
    }
}

#[derive(serde::Deserialize)]
struct AddListItemParamsBuilder {
    id: Option<Uuid>,
    list_type: String,
    item_name: String,
}

impl AddListItemParamsBuilder {
    fn build(self) -> Result<AddListItemParams, AddListItemParamsInvalid> {
        Ok(AddListItemParams::new(
            self.id,
            self.list_type,
            self.item_name,
        ))
    }
}

impl TryFrom<crate::JsonRpcRequest> for AddListItemParams {
    type Error = AddListItemParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: AddListItemParamsBuilder = serde_json::from_value(request.params)
            .map_err(AddListItemParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum AddListItemParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl Error for AddListItemParamsInvalid {}

impl Display for AddListItemParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            AddListItemParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AddListItemResult {
    pub success: bool,
    pub id: Option<Uuid>,
}

impl AddListItemResult {
    pub fn new(success: bool, id: Option<Uuid>) -> Self {
        Self { success, id }
    }
}
