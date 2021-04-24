use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize)]
#[non_exhaustive]
pub struct Params {
    pub id: Option<Uuid>,
    pub list_type: String,
    pub item_name: String,
}

impl Params {
    pub fn new(id: Option<Uuid>, list_type: String, item_name: String) -> Self {
        Self {
            id,
            list_type,
            item_name,
        }
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    id: Option<Uuid>,
    list_type: String,
    item_name: String,
}

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Ok(Params::new(self.id, self.list_type, self.item_name))
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

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub success: bool,
    pub id: Option<Uuid>,
}

impl MethodResult {
    pub fn new(success: bool, id: Option<Uuid>) -> Self {
        Self { success, id }
    }
}
