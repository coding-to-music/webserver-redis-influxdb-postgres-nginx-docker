use uuid::Uuid;

pub mod add_list_item;
pub mod delete_list_item;
pub mod get_list_items;
pub mod get_list_types;
pub mod rename_list_type;

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
