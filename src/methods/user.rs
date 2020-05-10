use crate::db;
use rand::SystemRandom;
use ring::{
    digest,
    rand::{self, SecureRandom},
};
use std::convert::TryInto;
use std::{num::NonZeroU32, sync::Arc};

pub struct UserController {
    db: Arc<db::Database<db::User>>,
}

impl UserController {
    pub fn new(db: Arc<db::Database<db::User>>) -> Self {
        Self { db }
    }

    pub async fn add<T: TryInto<add::AddUserParams, Error = add::AddUserParamsInvalid>>(
        &self,
        request: T,
    ) -> Result<add::AddUserResult, crate::Error> {
        let params: add::AddUserParams = request.try_into()?;

        if let Ok(Some(_)) = self.db.get_user(params.user().username()) {
            return Err(crate::Error::internal_error()
                .with_data("a user with that username already exists"));
        }

        let rng = SystemRandom::new();
        let mut salt = [0u8; digest::SHA512_OUTPUT_LEN];

        rng.fill(&mut salt).map_err(|e| {
            error!("{}", e);
            crate::Error::internal_error()
        })?;

        let hashed_password = self.encrypt(params.user().password(), &salt);

        let user_row = db::User::new(
            params.user().username().to_owned(),
            hashed_password.to_vec(),
            salt.to_vec(),
        );

        let rows = self.db.add_user(user_row)?;

        Ok(add::AddUserResult::new(rows == 1))
    }

    pub async fn change_password<
        T: TryInto<
            change_password::ChangePasswordParams,
            Error = change_password::ChangePasswordParamsInvalid,
        >,
    >(
        &self,
        request: T,
    ) -> Result<change_password::ChangePasswordResult, crate::Error> {
        let params: change_password::ChangePasswordParams = request.try_into()?;

        if self.db.validate_user(params.user()) {
            let user_row = self
                .db
                .get_user(params.user().username())?
                .ok_or_else(|| crate::Error::internal_error())?;
            let new_user_row = db::User::new(
                user_row.username().to_owned(),
                params.new_password().as_bytes().to_vec(),
                user_row.salt().to_vec(),
            );

            let result = self.db.update_user(new_user_row)?;
            Ok(change_password::ChangePasswordResult::new(result))
        } else {
            Ok(change_password::ChangePasswordResult::new(false))
        }
    }

    fn encrypt(
        &self,
        password: &str,
        salt: &[u8; digest::SHA512_OUTPUT_LEN],
    ) -> [u8; digest::SHA512_OUTPUT_LEN] {
        let mut hash = [0u8; digest::SHA512_OUTPUT_LEN];

        ring::pbkdf2::derive(
            ring::pbkdf2::PBKDF2_HMAC_SHA512,
            NonZeroU32::new(100_000).unwrap(),
            salt,
            password.as_bytes(),
            &mut hash,
        );

        hash
    }
}

#[derive(serde::Deserialize)]
pub struct User {
    username: String,
    password: String,
}

mod add {
    use super::*;
    use std::convert::TryFrom;

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
}

mod change_password {
    use super::*;
    use std::convert::TryFrom;

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
}
