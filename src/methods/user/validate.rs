use super::User;
use std::convert::{TryFrom, TryInto};

pub struct ValidateUserParams {
    user: User,
}

impl ValidateUserParams {
    pub fn user(&self) -> &User {
        &self.user
    }
}

#[derive(serde::Deserialize)]
pub struct ValidateUserParamsBuilder {
    user: User,
}

impl ValidateUserParamsBuilder {
    pub fn build(self) -> Result<ValidateUserParams, ValidateUserParamsInvalid> {
        Ok(ValidateUserParams { user: self.user })
    }
}

impl TryFrom<serde_json::Value> for ValidateUserParams {
    type Error = ValidateUserParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: ValidateUserParamsBuilder =
            serde_json::from_value(value).map_err(|_| ValidateUserParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for ValidateUserParams {
    type Error = ValidateUserParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

pub enum ValidateUserParamsInvalid {
    InvalidFormat,
}

impl From<ValidateUserParamsInvalid> for crate::Error {
    fn from(_: ValidateUserParamsInvalid) -> Self {
        crate::Error::invalid_params()
    }
}

#[derive(serde::Serialize)]
pub struct ValidateUserResult {
    valid: bool,
}

impl ValidateUserResult {
    pub fn new(valid: bool) -> Self {
        Self { valid }
    }
}
