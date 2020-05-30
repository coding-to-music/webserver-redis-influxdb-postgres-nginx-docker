use chrono::prelude::*;
use rand::SystemRandom;
use ring::{
    digest,
    rand::{self, SecureRandom},
};
use std::{convert::TryFrom, str::FromStr};

use crate::db::UserRole;
pub use controller::UserController;

mod controller;

#[derive(serde::Deserialize)]
pub struct User {
    username: String,
    password: String,
}

impl User {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

pub struct AddUserParams {
    user: User,
}

#[derive(serde::Deserialize)]
struct AddUserParamsBuilder {
    user: User,
}

impl AddUserParamsBuilder {
    fn build(self) -> Result<AddUserParams, AddUserParamsInvalid> {
        if self.user.password.len() < 10 {
            Err(AddUserParamsInvalid::PasswordTooShort)
        } else {
            Ok(AddUserParams { user: self.user })
        }
    }
}

pub enum AddUserParamsInvalid {
    InvalidFormat(serde_json::Error),
    PasswordTooShort,
}

impl TryFrom<crate::JsonRpcRequest> for AddUserParams {
    type Error = AddUserParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: AddUserParamsBuilder =
            serde_json::from_value(request.params).map_err(AddUserParamsInvalid::InvalidFormat)?;

        builder.build()
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

pub struct ChangePasswordParams {
    user: User,
    new_password: String,
}

#[derive(serde::Deserialize)]
struct ChangePasswordParamsBuilder {
    user: User,
    new_password: String,
}

impl ChangePasswordParamsBuilder {
    fn build(self) -> Result<ChangePasswordParams, ChangePasswordParamsInvalid> {
        Ok(ChangePasswordParams {
            user: self.user,
            new_password: self.new_password,
        })
    }
}

impl TryFrom<crate::JsonRpcRequest> for ChangePasswordParams {
    type Error = ChangePasswordParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ChangePasswordParamsBuilder = serde_json::from_value(request.params)
            .map_err(ChangePasswordParamsInvalid::InvalidFormat)?;

        builder.build()
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

pub struct ValidateUserParams {
    user: User,
}

#[derive(serde::Deserialize)]
struct ValidateUserParamsBuilder {
    user: User,
}

impl ValidateUserParamsBuilder {
    fn build(self) -> Result<ValidateUserParams, ValidateUserParamsInvalid> {
        Ok(ValidateUserParams { user: self.user })
    }
}

impl TryFrom<crate::JsonRpcRequest> for ValidateUserParams {
    type Error = ValidateUserParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ValidateUserParamsBuilder = serde_json::from_value(request.params)
            .map_err(ValidateUserParamsInvalid::InvalidFormat)?;

        builder.build()
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

pub struct SetRoleParams {
    user: User,
    username: String,
    role: UserRole,
}

#[derive(serde::Deserialize)]
struct SetRoleParamsBuilder {
    user: User,
    username: String,
    role: String,
}

impl SetRoleParamsBuilder {
    fn build(self) -> Result<SetRoleParams, SetRoleParamsInvalid> {
        let role = UserRole::from_str(&self.role).map_err(|_| SetRoleParamsInvalid::InvalidRole)?;
        Ok(SetRoleParams {
            user: self.user,
            username: self.username,
            role,
        })
    }
}

impl TryFrom<crate::JsonRpcRequest> for SetRoleParams {
    type Error = SetRoleParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: SetRoleParamsBuilder =
            serde_json::from_value(request.params).map_err(SetRoleParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

pub enum SetRoleParamsInvalid {
    InvalidFormat(serde_json::Error),
    InvalidRole,
}

impl From<SetRoleParamsInvalid> for crate::Error {
    fn from(error: SetRoleParamsInvalid) -> Self {
        match error {
            SetRoleParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
            SetRoleParamsInvalid::InvalidRole => {
                Self::invalid_params().with_data(format!("invalid role"))
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct SetRoleResult {
    success: bool,
}

impl SetRoleResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
