use super::ListItem;
use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub list_type: String,
}

impl Params {
    /// ## Error
    /// * If `list_type` is empty or whitespace.
    pub fn new(list_type: String) -> Result<Self, InvalidParams> {
        let trimmed = list_type.trim();
        if trimmed.is_empty() {
            Err(InvalidParams::ListTypeEmptyOrWhitespace)
        } else {
            Ok(Self {
                list_type: trimmed.to_owned(),
            })
        }
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
        Self::new(builder.list_type)
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    list_type: String,
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    ListTypeEmptyOrWhitespace,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            InvalidParams::ListTypeEmptyOrWhitespace => {
                crate::generic_invalid_value_message("list_type")
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub list_items: Vec<ListItem>,
}

impl MethodResult {
    pub fn new(list_items: Vec<ListItem>) -> Self {
        Self { list_items }
    }
}
