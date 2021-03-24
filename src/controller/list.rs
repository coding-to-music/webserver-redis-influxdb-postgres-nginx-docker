use crate::AppError;
use chrono::Utc;
use std::{collections::HashSet, convert::TryFrom, sync::Arc};
use uuid::Uuid;
use webserver_contracts::{list::*, JsonRpcError, JsonRpcRequest};
use webserver_database::{Database, ListItem as DbListItem};

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

        let created_s = Utc::now().timestamp() as u32;

        let new_item_id = params.id.unwrap_or_else(|| Uuid::new_v4());
        let list_type = params.list_type;
        let item_name = params.item_name;

        let result = self
            .db
            .insert_list_item(new_item_id, &list_type, &item_name, created_s)?;

        if result > 0 {
            Ok(AddListItemResult::new(true, Some(new_item_id)))
        } else {
            Ok(AddListItemResult::new(false, None))
        }
    }

    pub async fn get_list_items(
        &self,
        request: JsonRpcRequest,
    ) -> Result<GetListItemsResult, AppError> {
        let params = GetListItemsParams::try_from(request)?;

        let list_items = self.db.get_list_items(&params.list_type)?;

        let list_items = list_items
            .into_iter()
            .map(|li| ListItemWrapper::from(li).0)
            .collect();

        Ok(GetListItemsResult::new(list_items))
    }

    pub async fn delete_list_item(
        &self,
        request: JsonRpcRequest,
    ) -> Result<DeleteListItemResult, AppError> {
        let params = DeleteListItemParams::try_from(request)?;

        let id = params.id;

        info!("deleting list item with id '{}'", id);

        let result = self.db.delete_list_item(id)?;

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

impl From<AddListItemParamsInvalid> for AppError {
    fn from(error: AddListItemParamsInvalid) -> Self {
        match error {
            AddListItemParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}

impl From<GetListItemsParamsInvalid> for AppError {
    fn from(error: GetListItemsParamsInvalid) -> Self {
        match error {
            GetListItemsParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
            GetListItemsParamsInvalid::EmptyOrWhitespace => AppError::from(
                JsonRpcError::invalid_params()
                    .with_message("list_type must not be empty or whitespace"),
            ),
        }
    }
}

impl From<DeleteListItemParamsInvalid> for AppError {
    fn from(error: DeleteListItemParamsInvalid) -> Self {
        match error {
            DeleteListItemParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}

impl From<GetListTypesParamsInvalid> for AppError {
    fn from(error: GetListTypesParamsInvalid) -> Self {
        match error {
            GetListTypesParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}

impl From<RenameListTypeParamsInvalid> for AppError {
    fn from(error: RenameListTypeParamsInvalid) -> Self {
        match error {
            RenameListTypeParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
            RenameListTypeParamsInvalid::EmptyOldName => JsonRpcError::invalid_params()
                .with_message("'old_name' cannot be empty or whitespace")
                .into(),
            RenameListTypeParamsInvalid::EmptyNewName => JsonRpcError::invalid_params()
                .with_message("'new_name' cannot be empty or whitespace")
                .into(),
        }
    }
}

struct ListItemWrapper(ListItem);

impl From<DbListItem> for ListItemWrapper {
    fn from(db_list_item: DbListItem) -> Self {
        ListItemWrapper(ListItem::new(
            db_list_item.id,
            db_list_item.list_type,
            db_list_item.item_name,
        ))
    }
}
