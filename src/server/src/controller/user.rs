use crate::{
    app::{AppError, AppResult, ParamsError},
    auth::{Role, TokenHandler},
};
use database::{InsertionResult, User as DbUser, UserDatabase};
use model::{
    user::{add_user, get_token, get_user, User},
    JsonRpcError, JsonRpcRequest,
};
use std::{convert::TryFrom, sync::Arc};
use time::{ext::NumericalDuration, OffsetDateTime};
use uuid::Uuid;

pub struct UserController {
    user_db: Arc<UserDatabase>,
    token_handler: TokenHandler,
}

impl UserController {
    pub fn new(user_db: Arc<UserDatabase>, token_handler: TokenHandler) -> Self {
        Self {
            user_db,
            token_handler,
        }
    }

    pub async fn add_user(&self, request: JsonRpcRequest) -> AppResult<add_user::MethodResult> {
        use add_user::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let id = Uuid::new_v4().to_string();

        let result = self
            .user_db
            .insert_user(&id, &params.username, &params.password)
            .await?;

        match result {
            InsertionResult::Inserted => Ok(MethodResult::success(id)),
            InsertionResult::AlreadyExists => Ok(MethodResult::failure()),
        }
    }

    pub async fn get_token(&self, request: JsonRpcRequest) -> AppResult<get_token::MethodResult> {
        use get_token::{MethodResult, Params};
        let params = Params::try_from(request)?;
        if let Some(user) = self
            .user_db
            .validate_user(&params.username, &params.password)
            .await?
        {
            info!("{} successfully logged in", user.id);
            let roles = self.user_db.get_roles_for_user(&user.id).await?;
            let roles = roles
                .into_iter()
                .filter_map(|r| Role::from_sql_value(&r).ok())
                .collect();
            let exp = OffsetDateTime::now_utc().checked_add(1.hours()).ok_or(
                AppError::internal_error()
                    .with_context(&"failed to add 1 hour to current timestamp".to_string()),
            )?;
            let token = self.token_handler.generate_token(exp, roles);
            Ok(MethodResult::new(token))
        } else {
            Err(AppError::from(
                JsonRpcError::internal_error().with_message("invalid username or password"),
            ))
        }
    }

    pub async fn get_user(&self, request: JsonRpcRequest) -> AppResult<get_user::MethodResult> {
        use get_user::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let db_user = self.user_db.get_user_by_id(&params.id).await?;

        Ok(match db_user.map(|db_user| UserWrapper::from(db_user)) {
            Some(user) => MethodResult::exists(user.0),
            None => MethodResult::missing(),
        })
    }
}

impl ParamsError for add_user::InvalidParams {}
impl ParamsError for get_token::InvalidParams {}
impl ParamsError for get_user::InvalidParams {}

/// Used in order to convert from `database::User` to `model::User` (orphan rule).
struct UserWrapper(User);

impl From<DbUser> for UserWrapper {
    fn from(value: DbUser) -> Self {
        UserWrapper(User::new(value.id, value.username))
    }
}
