use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};

use chrono::Utc;
use uuid::Uuid;
use webserver_contracts::{list::*, Error as JsonRpcError, JsonRpcRequest};
use webserver_database::{Database, DatabaseError, ListItem as DbListItem};

use crate::AppError;

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

        let items_in_list = self.get_list_items_as_hash_map(&params.list_type)?;

        let new_item_id = params.id.unwrap_or_else(|| Uuid::new_v4());
        let list_type = params.list_type;
        let item_name = params.item_name;
        let next_better = params.next_better;
        let next_worse = params.next_worse;

        match (next_better, next_worse) {
            (None, None) if items_in_list.is_empty() => {
                // add item to new list
                self.db.insert_list_item(
                    new_item_id,
                    &list_type,
                    &item_name,
                    None,
                    None,
                    created_s,
                )?;
            }
            (None, None) => {
                return Err(JsonRpcError::application_error(-31999).with_message(format!("the list of type '{}' is not empty, so a new item must be better or worse than at least one other item", list_type)).into());
            }
            (None, Some(worse)) => {
                // this is the new best item
                // so the worse item should be the old best item
                let old_best = items_in_list
                    .get(&worse)
                    .ok_or(JsonRpcError::could_not_find_list_item(worse))?;
                if old_best.next_better.is_none() {
                    self.db.insert_list_item(
                        new_item_id,
                        &list_type,
                        &item_name,
                        next_better,
                        next_worse,
                        created_s,
                    )?;
                    // update the old best to point to the new best
                    self.db.update_list_item(
                        old_best.id,
                        &old_best.item_name,
                        Some(new_item_id),
                        old_best.next_worse,
                    )?;
                } else {
                    return Err(JsonRpcError::application_error(-31997).with_message("can't add a new best item if 'next_worse' does not point to the old best item").into());
                }
            }
            (Some(better), None) => {
                // this is the new worst item
                // so the better item should be the old worst item
                let old_worst = items_in_list
                    .get(&better)
                    .ok_or(JsonRpcError::could_not_find_list_item(better))?;
                if old_worst.next_worse.is_none() {
                    self.db.insert_list_item(
                        new_item_id,
                        &list_type,
                        &item_name,
                        next_better,
                        next_worse,
                        created_s,
                    )?;
                    // update the old worst to point to the new worst
                    self.db.update_list_item(
                        old_worst.id,
                        &old_worst.item_name,
                        old_worst.next_better,
                        Some(new_item_id),
                    )?;
                } else {
                    return Err(JsonRpcError::application_error(-31997).with_message("can't add a new worst item if 'next_best' does not point to the old worst item").into());
                }
            }
            (Some(better_id), Some(worse_id)) => {
                // this item goes in between two old items
                match (items_in_list.get(&better_id), items_in_list.get(&worse_id)) {
                    (None, None) => {
                        return Err(JsonRpcError::could_not_find_list_item(better_id).into());
                    }
                    (None, Some(_)) => {
                        return Err(JsonRpcError::could_not_find_list_item(better_id).into());
                    }
                    (Some(_), None) => {
                        return Err(JsonRpcError::could_not_find_list_item(worse_id).into());
                    }
                    (Some(better), Some(worse)) => {
                        self.db.insert_list_item(
                            new_item_id,
                            &list_type,
                            &item_name,
                            next_better,
                            next_worse,
                            created_s,
                        )?;

                        self.db.update_list_item(
                            better.id,
                            &better.item_name,
                            better.next_better,
                            Some(new_item_id),
                        )?;

                        self.db.update_list_item(
                            worse.id,
                            &worse.item_name,
                            Some(new_item_id),
                            worse.next_worse,
                        )?;
                    }
                }
            }
        }

        Ok(AddListItemResult::new(true, Some(new_item_id)))
    }

    pub async fn get_list_items(
        &self,
        request: JsonRpcRequest,
    ) -> Result<GetListItemsResult, AppError> {
        let params = GetListItemsParams::try_from(request)?;

        let list_items = self.db.get_list_items(&params.list_type)?;

        let list_items = list_items
            .into_iter()
            .map(|li| {
                ListItem::new(
                    li.id,
                    li.list_type,
                    li.item_name,
                    li.next_better,
                    li.next_worse,
                )
            })
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

        if let Some(item_to_delete) = self.db.get_list_item(id)? {
            let items_in_list = self.get_list_items_as_hash_map(&item_to_delete.list_type)?;

            let result = match (&item_to_delete.next_better, &item_to_delete.next_worse) {
                (None, None) => {
                    info!(
                        "list item with id '{}' is the only item in list '{}'",
                        id, item_to_delete.list_type
                    );
                    self.db.delete_list_item(id)?
                }
                (None, Some(worse)) => {
                    info!(
                        "list item with id '{}' is the best item in list '{}'",
                        id, item_to_delete.list_type
                    );
                    let worse_item = items_in_list.get(worse).unwrap();
                    self.db.update_list_item(
                        worse_item.id,
                        &worse_item.item_name,
                        None,
                        worse_item.next_worse,
                    )?;
                    self.db.delete_list_item(id)?
                }
                (Some(better), None) => {
                    info!(
                        "list item with id '{}' is the worst item in list '{}'",
                        id, item_to_delete.list_type
                    );
                    let better_item = items_in_list.get(better).unwrap();
                    self.db.update_list_item(
                        better_item.id,
                        &better_item.item_name,
                        better_item.next_better,
                        None,
                    )?;
                    self.db.delete_list_item(id)?
                }
                (Some(better), Some(worse)) => {
                    info!(
                        "list item with id '{}' is better than '{}' but worse than '{}' in list '{}'",
                        id,
                        worse,
                        better,
                        item_to_delete.list_type
                    );
                    let worse_item = items_in_list.get(worse).unwrap();
                    let better_item = items_in_list.get(better).unwrap();
                    self.db.update_list_item(
                        worse_item.id,
                        &worse_item.item_name,
                        item_to_delete.next_better,
                        worse_item.next_worse,
                    )?;
                    self.db.update_list_item(
                        better_item.id,
                        &better_item.item_name,
                        better_item.next_better,
                        item_to_delete.next_worse,
                    )?;
                    self.db.delete_list_item(id)?
                }
            };

            Ok(DeleteListItemResult::new(result))
        } else {
            Err(AppError::from(
                webserver_contracts::Error::application_error(-31998)
                    .with_message("item does not exist"),
            ))
        }
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

    fn get_list_items_as_hash_map(
        &self,
        list_type: &str,
    ) -> Result<HashMap<Uuid, DbListItem>, DatabaseError> {
        let items_in_list = self.db.get_list_items(list_type)?;

        Ok(items_in_list
            .into_iter()
            .map(|item| (item.id, item))
            .collect())
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
