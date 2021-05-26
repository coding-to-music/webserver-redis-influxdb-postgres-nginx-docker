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
    pub old_name: String,
    pub new_name: String,
}

impl Params {
    pub fn new(old_name: String, new_name: String) -> Result<Self, InvalidParams> {
        let old_trimmed = old_name.trim().to_owned();
        let new_trimmed = new_name.trim().to_owned();

        if old_trimmed.is_empty() {
            Err(InvalidParams::EmptyOldName)
        } else if new_trimmed.is_empty() {
            Err(InvalidParams::EmptyNewName)
        } else {
            Ok(Self {
                old_name: old_trimmed,
                new_name: new_trimmed,
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
        Self::new(builder.old_name, builder.new_name)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ParamsBuilder {
    old_name: String,
    new_name: String,
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    EmptyOldName,
    EmptyNewName,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            InvalidParams::EmptyOldName => crate::generic_invalid_value_message("old_name"),
            InvalidParams::EmptyNewName => crate::generic_invalid_value_message("new_name"),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub success: bool,
}

impl MethodResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
