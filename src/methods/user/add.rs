use super::User;
use std::convert::{TryFrom, TryInto};

pub struct AddUserParams {
    user: User,
}

impl AddUserParams {
    pub fn user(&self) -> &User {
        &self.user
    }
}

#[derive(serde::Deserialize)]
pub struct AddUserParamsBuilder {
    user: User,
}

impl AddUserParamsBuilder {
    pub fn build(self) -> Result<AddUserParams, AddUserParamsInvalid> {
        if self.user.password.len() < 10 {
            Err(AddUserParamsInvalid::PasswordTooShort)
        } else {
            Ok(AddUserParams { user: self.user })
        }
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
    InvalidFormat(serde_json::Error),
    PasswordTooShort,
}

impl TryFrom<serde_json::Value> for AddUserParams {
    type Error = AddUserParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: AddUserParamsBuilder =
            serde_json::from_value(value).map_err(AddUserParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for AddUserParams {
    type Error = AddUserParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

impl From<AddUserParamsInvalid> for crate::Error {
    fn from(error: AddUserParamsInvalid) -> Self {
        match error {
            AddUserParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
            AddUserParamsInvalid::PasswordTooShort => {
                Self::invalid_params().with_data("password is too short")
            }
        }
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
