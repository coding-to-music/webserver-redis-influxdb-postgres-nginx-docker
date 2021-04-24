use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize)]
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
            InvalidParams::InvalidName => "invalid tag name".to_string(),
            InvalidParams::InvalidValue => "invalid tag value".to_string(),
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

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Params::new(self.shape_id, self.name, self.value)
    }
}

impl TryFrom<crate::JsonRpcRequest> for Params {
    type Error = InvalidParams;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(request.params).map_err(InvalidParams::InvalidFormat)?;

        builder.build()
    }
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
