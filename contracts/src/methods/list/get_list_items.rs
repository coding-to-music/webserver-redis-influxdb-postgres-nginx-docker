use super::ListItem;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct Params {
    pub list_type: String,
}

impl Params {
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

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    list_type: String,
}

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Params::new(self.list_type)
    }
}

impl TryFrom<crate::JsonRpcRequest> for Params {
    type Error = InvalidParams;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder = serde_json::from_value(request.params)
            .map_err(InvalidParams::InvalidFormat)?;

        builder.build()
    }
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
                format!("'list_type' can not be empty or whitespace")
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
