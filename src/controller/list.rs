use std::{collections::HashMap, convert::TryFrom, sync::Arc};

use chrono::Utc;
use webserver_contracts::{
    list::{
        AddListItemParams, AddListItemParamsInvalid, AddListItemResult, GetListItemsParams,
        GetListItemsParamsInvalid, GetListItemsResult, ListItem,
    },
    Error as JsonRpcError, JsonRpcRequest,
};
use webserver_database::{Database, DatabaseError, ListItem as DbListItem};

use crate::AppError;

pub struct ListItemController {
    list_item_db: Arc<Database<DbListItem>>,
}

impl ListItemController {
    pub fn new(list_item_db: Arc<Database<DbListItem>>) -> Self {
        Self { list_item_db }
    }

    pub async fn add_list_item(
        &self,
        request: JsonRpcRequest,
    ) -> Result<AddListItemResult, AppError> {
        let params = AddListItemParams::try_from(request)?;

        let created_s = Utc::now().timestamp() as u32;

        let items_in_list = self.get_list_items_as_hash_map(params.list_item().list_type())?;

        let list_type = params.list_item().list_type();
        let item_name = params.list_item().item_name();
        let next_best = params.next_best();
        let next_worse = params.next_worse();

        match (next_best, next_worse) {
            (None, None) if items_in_list.is_empty() => {
                // add item to new list
                self.list_item_db()
                    .insert_list_item(list_type, item_name, None, None, created_s)
                    .unwrap();
            }
            (None, None) => {
                return Err(AppError::from(webserver_contracts::Error::application_error(-31999).with_message(format!("the list of type '{}' is not empty, so a new item must be better or worse than at least one other item", list_type))));
            }
            (None, Some(worse)) => {
                // this is the new best item
                // so the worse item should be the old best item
                if let Some(old_best) = items_in_list.get(worse) {
                    if old_best.next_best().is_none() {
                        self.list_item_db().insert_list_item(
                            list_type,
                            item_name,
                            *next_best,
                            *next_worse,
                            created_s,
                        )?;
                        let new_best = self
                            .list_item_db()
                            .get_list_item(list_type, item_name)?
                            .unwrap();
                        // update the old best to point to the new best
                        self.list_item_db().update_list_item(
                            *old_best.id(),
                            old_best.item_name(),
                            Some(*new_best.id()),
                            *old_best.next_worse(),
                        )?;
                    }
                }
            }
            (Some(better), None) => {
                // this is the new worst item
                // so the better item should be the old worst item
                if let Some(old_worst) = items_in_list.get(better) {
                    if old_worst.next_worse().is_none() {
                        self.list_item_db().insert_list_item(
                            list_type,
                            item_name,
                            *next_best,
                            *next_worse,
                            created_s,
                        )?;
                        let new_worst = self
                            .list_item_db()
                            .get_list_item(list_type, item_name)?
                            .unwrap();
                        // update the old worst to point to the new worst
                        self.list_item_db().update_list_item(
                            *old_worst.id(),
                            old_worst.item_name(),
                            *old_worst.next_best(),
                            Some(*new_worst.id()),
                        )?;
                    }
                }
            }
            (Some(better), Some(worse)) => {
                // this item goes in between two old items
                if let (Some(better), Some(worse)) =
                    (items_in_list.get(better), items_in_list.get(worse))
                {
                    self.list_item_db().insert_list_item(
                        list_type,
                        item_name,
                        *next_best,
                        *next_worse,
                        created_s,
                    )?;
                    let new_item = self
                        .list_item_db()
                        .get_list_item(list_type, item_name)?
                        .unwrap();

                    self.list_item_db().update_list_item(
                        *better.id(),
                        better.item_name(),
                        *better.next_best(),
                        Some(*new_item.id()),
                    )?;

                    self.list_item_db().update_list_item(
                        *worse.id(),
                        worse.item_name(),
                        Some(*new_item.id()),
                        *worse.next_worse(),
                    )?;
                }
            }
        }

        Ok(AddListItemResult::new(true))
    }

    pub async fn get_list_items(
        &self,
        request: JsonRpcRequest,
    ) -> Result<GetListItemsResult, AppError> {
        let params = GetListItemsParams::try_from(request)?;

        let list_items = self.list_item_db().get_list_items(params.list_type())?;

        let list_items = list_items
            .into_iter()
            .map(|li| ListItem::new(li.list_type().to_owned(), li.item_name().to_owned()))
            .collect();

        Ok(GetListItemsResult::new(list_items))
    }

    fn get_list_items_as_hash_map(
        &self,
        list_type: &str,
    ) -> Result<HashMap<u32, DbListItem>, DatabaseError> {
        let items_in_list = self.list_item_db.get_list_items(list_type)?;

        Ok(items_in_list
            .into_iter()
            .map(|item| (*item.id(), item))
            .collect())
    }

    /// Get a reference to the list item controller's list item db.
    pub fn list_item_db(&self) -> &Arc<Database<DbListItem>> {
        &self.list_item_db
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
