use crate::db;
use chrono::prelude::*;
use rand::SystemRandom;
use ring::{
    digest,
    rand::{self, SecureRandom},
};
use std::convert::TryInto;
use std::{num::NonZeroU32, sync::Arc};

mod add;
mod change_password;
mod validate;

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

        rng.fill(&mut salt)
            .map_err(|e| crate::Error::internal_error().with_data(format!("rng error: {}", e)))?;

        let hashed_password = Self::encrypt(params.user().password(), &salt);

        let user_row = db::User::new(
            params.user().username().to_owned(),
            hashed_password.to_vec(),
            salt.to_vec(),
            Utc::now().timestamp() as u32,
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

            let current_salt = user_row.salt();

            let new_password = Self::encrypt(params.new_password(), &current_salt);

            let new_user_row = db::User::new(
                user_row.username().to_owned(),
                new_password.to_vec(),
                current_salt.to_vec(),
                user_row.created_s(),
            );

            let result = self.db.update_user(new_user_row)?;
            Ok(change_password::ChangePasswordResult::new(result))
        } else {
            Ok(change_password::ChangePasswordResult::new(false))
        }
    }

    pub async fn validate_user<
        T: TryInto<validate::ValidateUserParams, Error = validate::ValidateUserParamsInvalid>,
    >(
        &self,
        request: T,
    ) -> Result<validate::ValidateUserResult, crate::Error> {
        let params: validate::ValidateUserParams = request.try_into()?;

        let result = self.db.validate_user(params.user());

        Ok(validate::ValidateUserResult::new(result))
    }

    fn encrypt(
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
