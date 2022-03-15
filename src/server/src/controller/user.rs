use crate::{
    app::{AppError, AppResult, ParamsError},
    auth::{Role, TokenHandler},
};
use database::{Database, InsertionResult, User, UserDatabase};
use model::{
    user::{add_user, get_token},
    JsonRpcError, JsonRpcRequest,
};
use std::{convert::TryFrom, str::FromStr, sync::Arc};
use uuid::Uuid;

pub struct UserController {
    user_db: Arc<UserDatabase>,
    token_handler: Arc<TokenHandler>,
}

impl UserController {
    pub fn new(user_db: Arc<Database<User>>, token_handler: Arc<TokenHandler>) -> Self {
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
                .filter_map(|r| Role::from_str(&r).ok())
                .collect();
            let exp = chrono::Utc::now()
                .checked_add_signed(chrono::Duration::seconds(3600))
                .unwrap()
                .timestamp();
            let token = self.token_handler.generate_token(exp, roles);
            Ok(MethodResult::new(token))
        } else {
            Err(AppError::from(
                JsonRpcError::internal_error().with_message("invalid username or password"),
            ))
        }
    }
}

impl ParamsError for add_user::InvalidParams {}
impl ParamsError for get_token::InvalidParams {}
