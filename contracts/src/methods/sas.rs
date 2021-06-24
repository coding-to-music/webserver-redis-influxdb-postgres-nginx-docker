use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct Params {
    pub key_name: String,
    pub key_value: String,
    pub resource_uri: String,
    pub weeks_expiry: u32,
}

impl Params {
    /// ## Error
    /// * If `key_name` is empty or whitespace.
    /// * If `key_value` is empty or whitespace.
    /// * If `resource_uri` is empty or whitespace.
    /// * If `weeks_expiry` is outside the range (1..=520).
    pub fn new(
        key_name: String,
        key_value: String,
        resource_uri: String,
        weeks_expiry: u32,
    ) -> Result<Self, InvalidParams> {
        use InvalidParams::*;
        let key_name = key_name.trim().to_owned();
        if key_name.is_empty() {
            return Err(InvalidKeyName);
        }

        let key_value = key_value.trim().to_owned();
        if key_value.is_empty() {
            return Err(InvalidKeyValue);
        }

        if !(1..=52 * 10).contains(&weeks_expiry) {
            return Err(InvalidWeeksExpiry);
        }

        Ok(Self {
            key_name,
            key_value,
            resource_uri,
            weeks_expiry,
        })
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(value: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(value.params).map_err(Self::Error::InvalidFormat)?;

        builder.try_into()
    }
}

impl TryFrom<ParamsBuilder> for Params {
    type Error = InvalidParams;

    fn try_from(builder: ParamsBuilder) -> Result<Self, Self::Error> {
        Self::new(
            builder.key_name,
            builder.key_value,
            builder.resource_uri,
            builder.weeks_expiry,
        )
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    key_name: String,
    key_value: String,
    resource_uri: String,
    weeks_expiry: u32,
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    InvalidWeeksExpiry,
    InvalidKeyName,
    InvalidKeyValue,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            InvalidParams::InvalidWeeksExpiry => {
                crate::generic_invalid_value_message("weeks_expiry")
            }
            InvalidParams::InvalidKeyName => crate::generic_invalid_value_message("key_name"),
            InvalidParams::InvalidKeyValue => crate::generic_invalid_value_message("key_value"),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub sas_key: String,
}

impl MethodResult {
    pub fn new(sas_key: String) -> Self {
        Self { sas_key }
    }
}
