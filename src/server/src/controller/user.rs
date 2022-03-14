use std::{convert::TryFrom, sync::Arc};

use chrono::Utc;
use database::{Database, InsertionResult, User, UserDatabase};
use model::{user::add_user, JsonRpcRequest};
use uuid::Uuid;

use crate::app::{AppResult, ParamsError};

pub struct UserController {
    user_db: Arc<UserDatabase>,
}

impl UserController {
    pub fn new(user_db: Arc<Database<User>>) -> Self {
        Self { user_db }
    }

    pub async fn add_user(&self, request: JsonRpcRequest) -> AppResult<add_user::MethodResult> {
        use add_user::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let created_s = Utc::now().timestamp();

        let id = Uuid::new_v4().to_string();

        let result = self
            .user_db
            .insert_user(&id, &params.username, &params.password, created_s)
            .await?;

        match result {
            InsertionResult::Inserted => Ok(MethodResult::success(id)),
            InsertionResult::AlreadyExists => Ok(MethodResult::failure()),
        }
    }
}

impl ParamsError for add_user::InvalidParams {}
