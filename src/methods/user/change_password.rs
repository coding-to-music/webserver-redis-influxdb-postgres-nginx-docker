use super::User;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize)]
pub struct ChangePasswordParams {
    user: User,
    new_password: String,
}

impl ChangePasswordParams {
    pub fn user(&self) -> &User {
        &self.user
    }

    pub fn new_password(&self) -> &str {
        &self.new_password
    }
}

impl TryFrom<serde_json::Value> for ChangePasswordParams {
    type Error = ChangePasswordParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let params = serde_json::from_value(value)
            .map_err(|_| ChangePasswordParamsInvalid::InvalidFormat)?;

        Ok(params)
    }
}

impl TryFrom<crate::JsonRpcRequest> for ChangePasswordParams {
    type Error = ChangePasswordParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

impl From<ChangePasswordParamsInvalid> for crate::Error {
    fn from(_: ChangePasswordParamsInvalid) -> Self {
        Self::invalid_params()
    }
}

pub enum ChangePasswordParamsInvalid {
    InvalidFormat,
}

#[derive(serde::Serialize)]
pub struct ChangePasswordResult {
    success: bool,
}

impl ChangePasswordResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
