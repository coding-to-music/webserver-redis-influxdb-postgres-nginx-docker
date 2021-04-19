use crate::{app::ParamsError, AppError};
use chrono::Utc;
use std::{collections::HashSet, convert::TryFrom, str::FromStr, sync::Arc};
use uuid::Uuid;
use webserver_contracts::{list::*, JsonRpcError, JsonRpcRequest};
use webserver_database::{Database, InsertionResult, ListItem as DbListItem};

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
    ) -> Result<AddListItemResult, AppError> {
        let params = AddListItemParams::try_from(request)?;

        let created_s = Utc::now().timestamp();

        let new_item_id = params.id.unwrap_or_else(|| Uuid::new_v4());
        let list_type = params.list_type;
        let item_name = params.item_name;

        let result = self.db.insert_list_item(
            &new_item_id.to_string(),
            &list_type,
            &item_name,
            created_s,
        )?;

        match result {
            InsertionResult::Inserted => Ok(AddListItemResult::new(true, Some(new_item_id))),
            InsertionResult::AlreadyExists => Ok(AddListItemResult::new(false, None)),
        }
    }

    pub async fn get_list_items(
        &self,
        request: JsonRpcRequest,
    ) -> Result<GetListItemsResult, AppError> {
        let params = GetListItemsParams::try_from(request)?;

        let list_items = self.db.get_list_items(&params.list_type)?;

        let list_items: Vec<ListItem> = list_items
            .into_iter()
            .map(|li| ListItemWrapper::try_from(li).and_then(|w| Ok(w.0)))
            .collect::<Result<_, _>>()?;

        Ok(GetListItemsResult::new(list_items))
    }

    pub async fn delete_list_item(
        &self,
        request: JsonRpcRequest,
    ) -> Result<DeleteListItemResult, AppError> {
        let params = DeleteListItemParams::try_from(request)?;

        let id = params.id.to_string();

        info!("deleting list item with id '{}'", id);

        let result = self.db.delete_list_item(&id)?;

        Ok(DeleteListItemResult::new(result))
    }

    pub async fn get_list_types(
        &self,
        request: JsonRpcRequest,
    ) -> Result<GetListTypesResult, AppError> {
        let _params = GetListTypesParams::try_from(request)?;

        let list_types = self.db.get_list_types()?;

        Ok(GetListTypesResult::new(list_types))
    }

    pub async fn rename_list_type(
        &self,
        request: JsonRpcRequest,
    ) -> Result<RenameListTypeResult, AppError> {
        let params = RenameListTypeParams::try_from(request)?;

        let existing_list_types: HashSet<_> = self.db.get_list_types()?.into_iter().collect();

        if !existing_list_types.contains(&params.old_name) {
            return Ok(RenameListTypeResult::new(false));
        }

        let updated_rows = self
            .db
            .rename_list_type(&params.old_name, &params.new_name)?;

        Ok(RenameListTypeResult::new(updated_rows > 0))
    }
}

struct ListItemWrapper(ListItem);

impl ParamsError for AddListItemParamsInvalid {}
impl ParamsError for GetListItemsParamsInvalid {}
impl ParamsError for DeleteListItemParamsInvalid {}
impl ParamsError for GetListTypesParamsInvalid {}
impl ParamsError for RenameListTypeParamsInvalid {}

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
