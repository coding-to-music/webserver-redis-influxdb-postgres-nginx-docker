use crate::AppError;
use chrono::Utc;
use db::{self, UserRole};
use rand::SystemRandom;
use ring::{
    digest,
    rand::{self, SecureRandom},
};
use std::{convert::TryFrom, str::FromStr, sync::Arc};
use webserver_contracts::{user::*, Error as JsonRpcError};

pub struct UserController {
    user_db: Arc<db::Database<db::User>>,
    prediction_db: Arc<db::Database<db::Prediction>>,
}

impl UserController {
    pub fn new(
        user_db: Arc<db::Database<db::User>>,
        prediction_db: Arc<db::Database<db::Prediction>>,
    ) -> Self {
        Self {
            user_db,
            prediction_db,
        }
    }

    pub async fn add(&self, request: crate::JsonRpcRequest) -> Result<AddUserResult, AppError> {
        let params = AddUserParams::try_from(request)?;

        if self.user_db.username_exists(&params.user().username()) {
            return Err(AppError::from(
                JsonRpcError::internal_error()
                    .with_data("a user with that username already exists"),
            ));
        }

        let rng = SystemRandom::new();
        let mut salt = [0u8; digest::SHA512_OUTPUT_LEN];

        rng.fill(&mut salt)
            .map_err(|e| JsonRpcError::internal_error().with_data(format!("rng error: {}", e)))?;

        let hashed_password = crate::encrypt(&params.user().password().as_bytes(), &salt);

        let user_row = db::User::new(
            None,
            params.user().username().to_owned(),
            hashed_password,
            salt,
            Utc::now().timestamp() as u32,
        );

        let rows = self.user_db.add_user(user_row)?;

        Ok(AddUserResult::new(rows == 1))
    }

    pub async fn change_password(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<ChangePasswordResult, AppError> {
        let params = ChangePasswordParams::try_from(request)?;

        let user_row = self.user_db.get_user(params.user().username())?;

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
            return Err(AppError::from(JsonRpcError::invalid_username_or_password()));
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

        let result = self.user_db.update_user_password(new_user_row)?;
        Ok(ChangePasswordResult::new(result))
    }

    pub async fn validate_user(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<ValidateUserResult, AppError> {
        let params = ValidateUserParams::try_from(request)?;
        let result = self
            .user_db
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
    ) -> Result<SetRoleResult, AppError> {
        let params = SetRoleParams::try_from(request)?;

        let user_row = self.user_db.get_user(params.user().username())?;

        if !user_row
            .as_ref()
            .map(|u| {
                let encrypted_password =
                    crate::encrypt(params.user().password().as_bytes(), u.salt());
                u.validate_password(&encrypted_password)
            })
            .unwrap_or(false)
        {
            return Err(AppError::from(JsonRpcError::invalid_username_or_password()));
        }

        let user_row = user_row.unwrap();

        if user_row.role() < &UserRole::Admin {
            return Err(AppError::from(JsonRpcError::not_permitted()));
        }

        if let Some(_user) = self.user_db.get_user(params.username())? {
            let result = self.user_db.update_user_role(
                params.username(),
                db::UserRole::from_str(params.role()).map_err(|_| {
                    AppError::from(JsonRpcError::invalid_params().with_data("invalid user role"))
                        .with_context(&format!(
                            "user provided '{}', which is not a valid role",
                            params.role()
                        ))
                })?,
            )?;
            Ok(SetRoleResult::new(result))
        } else {
            return Err(AppError::from(JsonRpcError::invalid_username_or_password()));
        }
    }

    pub async fn delete_user(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<DeleteUserResult, AppError> {
        let params = DeleteUserParams::try_from(request)?;

        let user_row = self.user_db.get_user(params.user().username())?;

        if !user_row
            .as_ref()
            .map(|u| {
                let encrypted_password =
                    crate::encrypt(params.user().password().as_bytes(), u.salt());
                u.validate_password(&encrypted_password)
            })
            .unwrap_or(false)
        {
            return Err(AppError::from(JsonRpcError::invalid_username_or_password()));
        }

        let user_row = user_row.unwrap();

        // only allow deletes if the user is an admin or if a user is trying to delete themselves
        if user_row.role() < &UserRole::Admin && params.user().username() != params.username() {
            return Err(AppError::from(JsonRpcError::not_permitted()));
        }

        let result = self.user_db.delete_user(params.username())?;

        if result {
            // delete any predictions associated with this user
            let deleted_predictions = self
                .prediction_db
                .delete_predictions_by_username(params.username())?;
            Ok(DeleteUserResult::new(result, deleted_predictions))
        } else {
            Err(AppError::from(JsonRpcError::database_error()))
        }
    }
}

impl From<AddUserParamsInvalid> for AppError {
    fn from(error: AddUserParamsInvalid) -> Self {
        match error {
            AddUserParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
            AddUserParamsInvalid::PasswordTooShort => {
                AppError::from(JsonRpcError::invalid_params().with_data("'password' too short"))
            }
        }
    }
}

impl From<ChangePasswordParamsInvalid> for AppError {
    fn from(error: ChangePasswordParamsInvalid) -> Self {
        match error {
            ChangePasswordParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}

impl From<ValidateUserParamsInvalid> for AppError {
    fn from(error: ValidateUserParamsInvalid) -> Self {
        match error {
            ValidateUserParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}

impl From<SetRoleParamsInvalid> for AppError {
    fn from(error: SetRoleParamsInvalid) -> Self {
        match error {
            SetRoleParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
            SetRoleParamsInvalid::InvalidRole => {
                AppError::from(JsonRpcError::invalid_params().with_data("invalid role"))
            }
        }
    }
}

impl From<DeleteUserParamsInvalid> for AppError {
    fn from(error: DeleteUserParamsInvalid) -> Self {
        match error {
            DeleteUserParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}
