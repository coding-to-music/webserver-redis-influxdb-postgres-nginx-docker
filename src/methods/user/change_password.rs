use super::User;
use std::convert::{TryFrom, TryInto};

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

#[derive(serde::Deserialize)]
pub struct ChangePasswordParamsBuilder {
    user: User,
    new_password: String,
}

impl ChangePasswordParamsBuilder {
    pub fn build(self) -> Result<ChangePasswordParams, ChangePasswordParamsInvalid> {
        Ok(ChangePasswordParams {
            user: self.user,
            new_password: self.new_password,
        })
    }
}

impl TryFrom<serde_json::Value> for ChangePasswordParams {
    type Error = ChangePasswordParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: ChangePasswordParamsBuilder =
            serde_json::from_value(value).map_err(ChangePasswordParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for ChangePasswordParams {
    type Error = ChangePasswordParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

impl From<ChangePasswordParamsInvalid> for crate::Error {
    fn from(error: ChangePasswordParamsInvalid) -> Self {
        match error {
            ChangePasswordParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
        }
    }
}

pub enum ChangePasswordParamsInvalid {
    InvalidFormat(serde_json::Error),
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
