use crate::JsonRpcRequest;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct Params {
    pub key_name: String,
    pub key_value: String,
    pub resource_uri: String,
    pub weeks_expiry: u32,
}

impl Params {
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

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    pub key_name: String,
    pub key_value: String,
    pub resource_uri: String,
    pub weeks_expiry: u32,
}

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Params::new(
            self.key_name,
            self.key_value,
            self.resource_uri,
            self.weeks_expiry,
        )
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;

    fn try_from(value: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(value.params).map_err(Self::Error::InvalidFormat)?;

        builder.build()
    }
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
            InvalidParams::InvalidWeeksExpiry => "invalid value of 'weeks_expiry'".to_string(),
            InvalidParams::InvalidKeyName => "invalid value of 'key_name'".to_string(),
            InvalidParams::InvalidKeyValue => "invalid value of 'key_value'".to_string(),
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
