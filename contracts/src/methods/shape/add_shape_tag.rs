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
    pub shape_id: Uuid,
    pub name: String,
    pub value: String,
}

impl Params {
    pub fn new(shape_id: Uuid, name: String, value: String) -> Result<Self, InvalidParams> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(InvalidParams::InvalidName);
        }

        let trimmed_value = value.trim();
        if trimmed_value.is_empty() {
            return Err(InvalidParams::InvalidValue);
        }

        let name = trimmed_name.to_owned();
        let value = trimmed_value.to_owned();

        Ok(Self {
            shape_id,
            name,
            value,
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
        Params::new(builder.shape_id, builder.name, builder.value)
    }
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    InvalidName,
    InvalidValue,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(e) => crate::invalid_params_serde_message(e),
            InvalidParams::InvalidName => crate::generic_invalid_value_message("name"),
            InvalidParams::InvalidValue => crate::generic_invalid_value_message("value"),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    shape_id: Uuid,
    name: String,
    value: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub success: bool,
    pub id: Option<String>,
}

impl MethodResult {
    fn new(success: bool, id: Option<String>) -> Self {
        Self { success, id }
    }

    pub fn success(id: String) -> Self {
        Self::new(true, Some(id))
    }

    pub fn failure() -> Self {
        Self::new(false, None)
    }
}
