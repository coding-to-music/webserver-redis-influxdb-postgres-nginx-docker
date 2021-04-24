use super::ListItem;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct GetListItemsParams {
    pub list_type: String,
}

impl GetListItemsParams {
    pub fn new(list_type: String) -> Result<Self, GetListItemsParamsInvalid> {
        let trimmed = list_type.trim();
        if trimmed.is_empty() {
            Err(GetListItemsParamsInvalid::ListTypeEmptyOrWhitespace)
        } else {
            Ok(Self {
                list_type: trimmed.to_owned(),
            })
        }
    }
}

#[derive(serde::Deserialize)]
struct GetListItemsParamsBuilder {
    list_type: String,
}

impl GetListItemsParamsBuilder {
    fn build(self) -> Result<GetListItemsParams, GetListItemsParamsInvalid> {
        GetListItemsParams::new(self.list_type)
    }
}

impl TryFrom<crate::JsonRpcRequest> for GetListItemsParams {
    type Error = GetListItemsParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: GetListItemsParamsBuilder = serde_json::from_value(request.params)
            .map_err(GetListItemsParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum GetListItemsParamsInvalid {
    InvalidFormat(serde_json::Error),
    ListTypeEmptyOrWhitespace,
}

impl Error for GetListItemsParamsInvalid {}

impl Display for GetListItemsParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            GetListItemsParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            GetListItemsParamsInvalid::ListTypeEmptyOrWhitespace => {
                format!("'list_type' can not be empty or whitespace")
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GetListItemsResult {
    pub list_items: Vec<ListItem>,
}

impl GetListItemsResult {
    pub fn new(list_items: Vec<ListItem>) -> Self {
        Self { list_items }
    }
}
