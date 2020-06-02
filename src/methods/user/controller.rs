use super::*;
use crate::db;
use std::{convert::TryInto, sync::Arc};

pub struct UserController {
    db: Arc<db::Database<db::User>>,
}

impl UserController {
    pub fn new(db: Arc<db::Database<db::User>>) -> Self {
        Self { db }
    }

    pub async fn add(&self, request: crate::JsonRpcRequest) -> Result<AddUserResult, crate::Error> {
        let params: AddUserParams = request.try_into()?;

        if self.db.username_exists(&params.user().username()) {
            return Err(crate::Error::internal_error()
                .with_data("a user with that username already exists"));
        }

        let rng = SystemRandom::new();
        let mut salt = [0u8; digest::SHA512_OUTPUT_LEN];

        rng.fill(&mut salt)
            .map_err(|e| crate::Error::internal_error().with_data(format!("rng error: {}", e)))?;

        let hashed_password = db::encrypt(&params.user().password().as_bytes(), &salt);

        let user_row = db::User::new(
            None,
            params.user().username().to_owned(),
            hashed_password,
            salt,
            Utc::now().timestamp() as u32,
        );

        let rows = self.db.add_user(user_row)?;

        Ok(AddUserResult::new(rows == 1))
    }

    pub async fn change_password(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<ChangePasswordResult, crate::Error> {
        let params: ChangePasswordParams = request.try_into()?;

        if self.get_and_validate(params.user()) {
            let user_row = self
                .db
                .get_user(params.user().username())?
                .ok_or_else(crate::Error::internal_error)?;

            let current_salt = user_row.salt();

            let new_password = db::encrypt(params.new_password().as_bytes(), &current_salt);

            let new_user_row = db::User::new(
                user_row.id(),
                user_row.username().to_owned(),
                new_password,
                current_salt,
                user_row.created_s(),
            );

            let result = self.db.update_user_password(new_user_row)?;
            Ok(ChangePasswordResult::new(result))
        } else {
            Ok(ChangePasswordResult::new(false))
        }
    }

    pub async fn validate_user(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<ValidateUserResult, crate::Error> {
        let params: ValidateUserParams = request.try_into()?;

        let result = self.get_and_validate(params.user());

        Ok(ValidateUserResult::new(result))
    }

    pub async fn set_role(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<SetRoleResult, crate::Error> {
        let params: SetRoleParams = request.try_into()?;

        if !self.get_and_validate(params.user()) {
            return Err(crate::Error::internal_error().with_data("invalid username or password"));
        }

        if self
            .db
            .get_user(params.user().username())?
            .map(|user| *user.role())
            .unwrap_or(UserRole::User)
            < UserRole::Admin
        {
            return Err(crate::Error::internal_error().with_data("you do not have permission"));
        }

        if let Some(_user) = self.db.get_user(params.username())? {
            let result = self.db.update_user_role(params.username(), params.role())?;
            Ok(SetRoleResult::new(result))
        } else {
            Err(crate::Error::internal_error().with_data("user does not exist"))
        }
    }

    fn get_and_validate(&self, user: &crate::methods::User) -> bool {
        self.db
            .get_user(user.username())
            .unwrap_or(None)
            .map(|u| u.validate_password(user.password().as_bytes()))
            .unwrap_or(false)
    }
}
