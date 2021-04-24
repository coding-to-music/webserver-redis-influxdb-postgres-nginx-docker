use crate::JsonRpcRequest;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct SleepParams {
    pub ms: u64,
}

impl SleepParams {
    pub fn new(ms: u64) -> Result<Self, SleepParamsInvalid> {
        if ms > 10_000 {
            Err(SleepParamsInvalid::InvalidDuration)
        } else {
            Ok(Self { ms })
        }
    }
}

#[derive(serde::Deserialize)]
struct SleepParamsBuilder {
    pub ms: u64,
}

impl SleepParamsBuilder {
    fn build(self) -> Result<SleepParams, SleepParamsInvalid> {
        SleepParams::new(self.ms)
    }
}

impl TryFrom<JsonRpcRequest> for SleepParams {
    type Error = SleepParamsInvalid;

    fn try_from(value: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: SleepParamsBuilder =
            serde_json::from_value(value.params).map_err(Self::Error::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum SleepParamsInvalid {
    InvalidFormat(serde_json::Error),
    InvalidDuration,
}

impl Error for SleepParamsInvalid {}

impl Display for SleepParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            SleepParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            SleepParamsInvalid::InvalidDuration => "'duration' has an invalid value".to_string(),
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SleepResult {
    pub slept_ms: u64,
}

impl SleepResult {
    pub fn new(slept_ms: u64) -> Self {
        Self { slept_ms }
    }
}
