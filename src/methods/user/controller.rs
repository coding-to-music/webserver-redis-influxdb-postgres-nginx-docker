use super::*;
use crate::db;
use std::num::NonZeroU32;
use std::{convert::TryInto, sync::Arc};

pub struct UserController {
    db: Arc<db::Database<db::User>>,
}

impl UserController {
    pub fn new(db: Arc<db::Database<db::User>>) -> Self {
        Self { db }
    }

    pub async fn add<T: TryInto<AddUserParams, Error = AddUserParamsInvalid>>(
        &self,
        request: T,
    ) -> Result<AddUserResult, crate::Error> {
        let params: AddUserParams = request.try_into()?;

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
            None,
            params.user().username().to_owned(),
            hashed_password.to_vec(),
            salt.to_vec(),
            Utc::now().timestamp() as u32,
        );

        let rows = self.db.add_user(user_row)?;

        Ok(AddUserResult::new(rows == 1))
    }

    pub async fn change_password<
        T: TryInto<ChangePasswordParams, Error = ChangePasswordParamsInvalid>,
    >(
        &self,
        request: T,
    ) -> Result<ChangePasswordResult, crate::Error> {
        let params: ChangePasswordParams = request.try_into()?;

        if self.db.validate_user(params.user()) {
            let user_row = self
                .db
                .get_user(params.user().username())?
                .ok_or_else(crate::Error::internal_error)?;

            let current_salt = user_row.salt();

            let new_password = Self::encrypt(params.new_password(), &current_salt);

            let new_user_row = db::User::new(
                user_row.id(),
                user_row.username().to_owned(),
                new_password.to_vec(),
                current_salt.to_vec(),
                user_row.created_s(),
            );

            let result = self.db.update_user(new_user_row)?;
            Ok(ChangePasswordResult::new(result))
        } else {
            Ok(ChangePasswordResult::new(false))
        }
    }

    pub async fn validate_user<
        T: TryInto<ValidateUserParams, Error = ValidateUserParamsInvalid>,
    >(
        &self,
        request: T,
    ) -> Result<ValidateUserResult, crate::Error> {
        let params: ValidateUserParams = request.try_into()?;

        let result = self.db.validate_user(params.user());

        Ok(ValidateUserResult::new(result))
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