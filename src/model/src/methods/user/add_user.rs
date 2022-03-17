use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};

pub const PASSWORD_MIN_LEN: usize = 10;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub username: String,
    pub password: String,
}

impl Params {
    pub fn new(username: String, password: String) -> Result<Self, InvalidParams> {
        if password.len() < PASSWORD_MIN_LEN {
            return Err(InvalidParams::PasswordTooShort);
        }

        Ok(Self { username, password })
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
        Params::new(builder.username, builder.password)
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    username: String,
    password: String,
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    PasswordTooShort,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            InvalidParams::PasswordTooShort => crate::invalid_value_because_message(
                "password",
                format!("must be at least {} characters long", PASSWORD_MIN_LEN),
            ),
        };
        write!(f, "{}", output)
    }
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub success: bool,
    pub id: Option<String>,
}

impl MethodResult {
    pub fn success(id: String) -> Self {
        Self {
            success: true,
            id: Some(id),
        }
    }

    pub fn failure() -> Self {
        Self {
            success: false,
            id: None,
        }
    }
}
