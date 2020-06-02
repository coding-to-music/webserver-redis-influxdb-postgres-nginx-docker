use chrono::Utc;
use db::{self, UserRole};
use rand::SystemRandom;
use ring::{
    digest,
    rand::{self, SecureRandom},
};
use std::{convert::TryFrom, sync::Arc};
use webserver_contracts::user::*;

pub struct UserController {
    db: Arc<db::Database<db::User>>,
}

impl UserController {
    pub fn new(db: Arc<db::Database<db::User>>) -> Self {
        Self { db }
    }

    pub async fn add(&self, request: crate::JsonRpcRequest) -> Result<AddUserResult, crate::Error> {
        let params = AddUserParams::try_from(request)?;

        if self.db.username_exists(&params.user().username()) {
            return Err(crate::Error::internal_error()
                .with_data("a user with that username already exists"));
        }

        let rng = SystemRandom::new();
        let mut salt = [0u8; digest::SHA512_OUTPUT_LEN];

        rng.fill(&mut salt)
            .map_err(|e| crate::Error::internal_error().with_data(format!("rng error: {}", e)))?;

        let hashed_password = crate::encrypt(&params.user().password().as_bytes(), &salt);

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
        let params = ChangePasswordParams::try_from(request)?;

        let user_row = self.db.get_user(params.user().username())?;

        // check that the user exists and that the password is valid
        if !user_row
            .as_ref()
            .map(|u| {
                let encrypted_password =
                    crate::encrypt(params.user().password().as_bytes(), u.salt());
                u.validate_password(&encrypted_password)
            })
            .unwrap_or(false)
        {
            return Err(crate::Error::invalid_username_or_password());
        }

        let user_row = user_row.unwrap(); // always Some because we did unwrap_or(false) above

        let current_salt = user_row.salt();

        let new_password = crate::encrypt(params.new_password().as_bytes(), current_salt);

        let new_user_row = db::User::new(
            user_row.id(),
            user_row.username().to_owned(),
            new_password,
            *current_salt,
            user_row.created_s(),
        );

        let result = self.db.update_user_password(new_user_row)?;
        Ok(ChangePasswordResult::new(result))
    }

    pub async fn validate_user(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<ValidateUserResult, crate::Error> {
        let params = ValidateUserParams::try_from(request)?;
        let result = self
            .db
            .get_user(params.user().username())?
            .map(|u| {
                let encrypted_password =
                    crate::encrypt(params.user().password().as_bytes(), u.salt());
                u.validate_password(&encrypted_password)
            })
            .unwrap_or(false);

        Ok(ValidateUserResult::new(result))
    }

    pub async fn set_role(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<SetRoleResult, crate::Error> {
        let params = SetRoleParams::try_from(request)?;

        let user_row = self.db.get_user(params.user().username())?;

        if !user_row
            .as_ref()
            .map(|u| {
                let encrypted_password =
                    crate::encrypt(params.user().password().as_bytes(), u.salt());
                u.validate_password(&encrypted_password)
            })
            .unwrap_or(false)
        {
            return Err(crate::Error::invalid_username_or_password());
        }

        let user_row = user_row.unwrap();

        if user_row.role() < &UserRole::Admin {
            return Err(crate::Error::not_permitted());
        }

        if let Some(_user) = self.db.get_user(params.username())? {
            let result = self.db.update_user_role(params.username(), params.role())?;
            Ok(SetRoleResult::new(result))
        } else {
            Err(crate::Error::internal_error().with_data("user does not exist"))
        }
    }

    pub async fn delete_user(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<DeleteUserResult, crate::Error> {
        let params = DeleteUserParams::try_from(request)?;

        let user_row = self.db.get_user(params.user().username())?;

        if !user_row
            .as_ref()
            .map(|u| {
                let encrypted_password =
                    crate::encrypt(params.user().password().as_bytes(), u.salt());
                u.validate_password(&encrypted_password)
            })
            .unwrap_or(false)
        {
            return Err(crate::Error::invalid_username_or_password());
        }

        let user_row = user_row.unwrap();

        if user_row.role() < &UserRole::Admin {
            return Err(crate::Error::not_permitted());
        }

        let result = self.db.delete_user(params.username())?;

        Ok(DeleteUserResult::new(result))
    }
}
