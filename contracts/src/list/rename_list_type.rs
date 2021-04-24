use crate::JsonRpcRequest;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Debug, Clone, serde::Serialize)]
#[non_exhaustive]
pub struct RenameListTypeParams {
    pub old_name: String,
    pub new_name: String,
}

impl RenameListTypeParams {
    pub fn new(old_name: String, new_name: String) -> Result<Self, RenameListTypeParamsInvalid> {
        let old_trimmed = old_name.trim().to_owned();
        let new_trimmed = new_name.trim().to_owned();

        if old_trimmed.is_empty() {
            Err(RenameListTypeParamsInvalid::EmptyOldName)
        } else if new_trimmed.is_empty() {
            Err(RenameListTypeParamsInvalid::EmptyNewName)
        } else {
            Ok(Self {
                old_name: old_trimmed,
                new_name: new_trimmed,
            })
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RenameListTypeParamsBuilder {
    old_name: String,
    new_name: String,
}

impl RenameListTypeParamsBuilder {
    fn build(self) -> Result<RenameListTypeParams, RenameListTypeParamsInvalid> {
        RenameListTypeParams::new(self.old_name, self.new_name)
    }
}

impl TryFrom<JsonRpcRequest> for RenameListTypeParams {
    type Error = RenameListTypeParamsInvalid;

    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: RenameListTypeParamsBuilder = serde_json::from_value(request.params)
            .map_err(RenameListTypeParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum RenameListTypeParamsInvalid {
    InvalidFormat(serde_json::Error),
    EmptyOldName,
    EmptyNewName,
}

impl Error for RenameListTypeParamsInvalid {}

impl Display for RenameListTypeParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            RenameListTypeParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            RenameListTypeParamsInvalid::EmptyOldName => {
                "'old_name' can not be empty or whitespace".to_string()
            }
            RenameListTypeParamsInvalid::EmptyNewName => {
                "'new_name' can not be empty or whitespace".to_string()
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RenameListTypeResult {
    pub success: bool,
}

impl RenameListTypeResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
