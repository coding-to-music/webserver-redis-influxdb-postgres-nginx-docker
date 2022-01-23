use crate::{
    app::{AppResult, ParamsError},
    AppError,
};
use chrono::Utc;
use database::{Database, InsertionResult, ListItem as DbListItem};
use model::{list::*, JsonRpcError, JsonRpcRequest};
use std::{collections::HashSet, convert::TryFrom, str::FromStr, sync::Arc};
use uuid::Uuid;

pub struct ListItemController {
    db: Arc<Database<DbListItem>>,
}

impl ListItemController {
    pub fn new(list_item_db: Arc<Database<DbListItem>>) -> Self {
        Self { db: list_item_db }
    }

    pub async fn add_list_item(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<add_list_item::MethodResult> {
        use add_list_item::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let created_s = Utc::now().timestamp();

        let new_item_id = params.id.unwrap_or_else(Uuid::new_v4);
        let list_type = params.list_type;
        let item_name = params.item_name;

        let result = self
            .db
            .insert_list_item(&new_item_id.to_string(), &list_type, &item_name, created_s)
            .await?;

        match result {
            InsertionResult::Inserted => Ok(MethodResult::new(true, Some(new_item_id))),
            InsertionResult::AlreadyExists => Ok(MethodResult::new(false, None)),
        }
    }

    pub async fn get_list_items(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<get_list_items::MethodResult> {
        use get_list_items::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let list_items = self.db.get_list_items(&params.list_type).await?;

        let list_items: Vec<ListItem> = list_items
            .into_iter()
            .map(|li| ListItemWrapper::try_from(li).map(|w| w.0))
            .collect::<Result<_, _>>()?;

        Ok(MethodResult::new(list_items))
    }

    pub async fn delete_list_item(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<delete_list_item::MethodResult> {
        use delete_list_item::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let id = params.id.to_string();

        info!("deleting list item with id '{}'", id);

        let result = self.db.delete_list_item(&id).await?;

        Ok(MethodResult::new(result))
    }

    pub async fn get_list_types(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<get_list_types::MethodResult> {
        use get_list_types::{MethodResult, Params};
        let _params = Params::try_from(request)?;

        let list_types = self.db.get_list_types().await?;

        Ok(MethodResult::new(list_types))
    }

    pub async fn rename_list_type(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<rename_list_type::MethodResult> {
        use rename_list_type::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let existing_list_types: HashSet<_> = self.db.get_list_types().await?.into_iter().collect();

        if !existing_list_types.contains(&params.old_name) {
            return Ok(MethodResult::new(false));
        }

        let updated_rows = self
            .db
            .rename_list_type(&params.old_name, &params.new_name)
            .await?;

        Ok(MethodResult::new(updated_rows > 0))
    }
}

struct ListItemWrapper(ListItem);

impl TryFrom<DbListItem> for ListItemWrapper {
    type Error = AppError;

    fn try_from(db_list_item: DbListItem) -> Result<Self, Self::Error> {
        let id = Uuid::from_str(&db_list_item.id)
            .map_err(|e| AppError::from(JsonRpcError::internal_error()).with_context(&e))?;

        Ok(ListItemWrapper(ListItem::new(
            id,
            db_list_item.list_type,
            db_list_item.item_name,
        )))
    }
}

impl ParamsError for add_list_item::InvalidParams {}
impl ParamsError for get_list_items::InvalidParams {}
impl ParamsError for delete_list_item::InvalidParams {}
impl ParamsError for get_list_types::InvalidParams {}
impl ParamsError for rename_list_type::InvalidParams {}
