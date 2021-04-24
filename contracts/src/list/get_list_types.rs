use crate::JsonRpcRequest;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Debug, Clone, serde::Serialize)]
#[non_exhaustive]
pub struct GetListTypesParams {}

impl GetListTypesParams {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(serde::Deserialize)]
struct GetListTypesParamsBuilder {}

impl GetListTypesParamsBuilder {
    fn build(self) -> Result<GetListTypesParams, GetListTypesParamsInvalid> {
        Ok(GetListTypesParams::new())
    }
}

impl TryFrom<JsonRpcRequest> for GetListTypesParams {
    type Error = GetListTypesParamsInvalid;

    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: GetListTypesParamsBuilder = serde_json::from_value(request.params)
            .map_err(GetListTypesParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(Debug)]
pub enum GetListTypesParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl Error for GetListTypesParamsInvalid {}

impl Display for GetListTypesParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            GetListTypesParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GetListTypesResult {
    pub list_types: Vec<String>,
}

impl GetListTypesResult {
    pub fn new(list_types: Vec<String>) -> Self {
        Self { list_types }
    }
}
