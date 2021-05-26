use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub id: Option<Uuid>,
    pub list_type: String,
    pub item_name: String,
}

impl Params {
    pub fn new(
        id: Option<Uuid>,
        list_type: String,
        item_name: String,
    ) -> Result<Self, InvalidParams> {
        use InvalidParams::*;

        let list_type_trimmed = list_type.trim();
        if list_type_trimmed.is_empty() {
            return Err(InvalidListType);
        }

        let item_name_trimmed = item_name.trim();
        if item_name_trimmed.is_empty() {
            return Err(InvalidItemName);
        }

        Ok(Self {
            id,
            list_type: list_type_trimmed.to_owned(),
            item_name: item_name_trimmed.to_owned(),
        })
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
        Self::new(builder.id, builder.list_type, builder.item_name)
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    id: Option<Uuid>,
    list_type: String,
    item_name: String,
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    InvalidListType,
    InvalidItemName,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            InvalidParams::InvalidListType => crate::generic_invalid_value_message("list_type"),
            InvalidParams::InvalidItemName => crate::generic_invalid_value_message("item_name"),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub success: bool,
    pub id: Option<Uuid>,
}

impl MethodResult {
    pub fn new(success: bool, id: Option<Uuid>) -> Self {
        Self { success, id }
    }
}
