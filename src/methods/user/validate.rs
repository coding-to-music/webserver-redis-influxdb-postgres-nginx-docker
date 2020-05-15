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
            serde_json::from_value(value).map_err(ValidateUserParamsInvalid::InvalidFormat)?;

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
    InvalidFormat(serde_json::Error),
}

impl From<ValidateUserParamsInvalid> for crate::Error {
    fn from(error: ValidateUserParamsInvalid) -> Self {
        match error {
            ValidateUserParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
        }
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
