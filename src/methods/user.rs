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
