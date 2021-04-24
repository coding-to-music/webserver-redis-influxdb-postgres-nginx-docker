pub use add_list_item::*;
pub use delete_list_item::*;
pub use get_list_items::*;
pub use get_list_types::*;
pub use rename_list_type::*;

use uuid::Uuid;

mod add_list_item;
mod delete_list_item;
mod get_list_items;
mod get_list_types;
mod rename_list_type;

#[derive(serde::Serialize, Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct ListItem {
    pub id: Uuid,
    pub list_type: String,
    pub item_name: String,
}

impl ListItem {
    pub fn new(id: Uuid, list_type: String, item_name: String) -> Self {
        Self {
            id,
            list_type,
            item_name,
        }
    }
}
