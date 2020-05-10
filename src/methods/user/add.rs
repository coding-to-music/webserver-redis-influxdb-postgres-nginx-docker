use super::User;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize)]
pub struct AddUserParams {
    user: User,
}

impl AddUserParams {
    pub fn user(&self) -> &User {
        &self.user
    }
}

impl User {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

pub enum AddUserParamsInvalid {
    InvalidFormat,
    PasswordTooShort,
}

impl TryFrom<serde_json::Value> for AddUserParams {
    type Error = AddUserParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let params: AddUserParams =
            serde_json::from_value(value).map_err(|_| AddUserParamsInvalid::InvalidFormat)?;

        if params.user.password.len() < 10 {
            Err(AddUserParamsInvalid::PasswordTooShort)
        } else {
            Ok(params)
        }
    }
}

impl TryFrom<crate::JsonRpcRequest> for AddUserParams {
    type Error = AddUserParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

impl From<AddUserParamsInvalid> for crate::Error {
    fn from(_: AddUserParamsInvalid) -> Self {
        Self::invalid_params()
    }
}

#[derive(serde::Serialize)]
pub struct AddUserResult {
    success: bool,
}

impl AddUserResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
