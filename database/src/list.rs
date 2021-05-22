use crate::{Database, DatabaseResult, InsertionResult};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Row};
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListItem {
    pub id: String,
    pub list_type: String,
    pub item_name: String,
    pub created_s: i64,
}

impl ListItem {
    fn new(id: String, list_type: String, item_name: String, created_s: i64) -> Self {
        Self {
            id,
            list_type,
            item_name,
            created_s,
        }
    }

    pub fn created_utc(&self) -> DateTime<Utc> {
        chrono::Utc.timestamp(self.created_s, 0)
    }
}

impl<'a> TryFrom<&Row<'a>> for ListItem {
    type Error = rusqlite::Error;
    fn try_from(row: &Row<'a>) -> Result<Self, Self::Error> {
        let id: String = row.get(0)?;
        let list_type: String = row.get(1)?;
        let item_name: String = row.get(2)?;
        let created_s = row.get(3)?;

        Ok(ListItem::new(id, list_type, item_name, created_s))
    }
}

impl Database<ListItem> {
    pub fn insert_list_item(
        &self,
        id: &str,
        list_type: &str,
        item_name: &str,
        created_s: i64,
    ) -> DatabaseResult<InsertionResult> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO list_item (id, list_type, item_name, created_s) VALUES (?1, ?2, ?3, ?4)",
            params![id, list_type, item_name, created_s],
        )?;

        Ok(InsertionResult::from_changed_rows(changed_rows))
    }

    pub fn get_list_item(&self, id: &str) -> DatabaseResult<Option<ListItem>> {
        let db = self.get_connection()?;

        let mut stmt =
            db.prepare("SELECT id, list_type, item_name, created_s FROM list_item WHERE id = ?1")?;

        let mut list_item_rows: Vec<_> = stmt
            .query_map(params![id], |row| crate::parse_from_row(row))?
            .collect::<Result<_, _>>()?;

        if list_item_rows.is_empty() {
            Ok(None)
        } else if list_item_rows.len() > 1 {
            error!(r#"more than 1 list item with id: "{}""#, id);
            Ok(None)
        } else {
            Ok(Some(list_item_rows.swap_remove(0)))
        }
    }

    pub fn get_list_types(&self) -> DatabaseResult<Vec<String>> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare("SELECT DISTINCT list_type FROM list_item")?;

        let user_rows: Vec<String> = stmt
            .query_map(params![], |row| row.get(0))?
            .collect::<Result<_, _>>()?;

        Ok(user_rows)
    }

    pub fn update_list_item(&self, id: &str, item_name: &str) -> DatabaseResult<usize> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "UPDATE list_item SET item_name = ?1 WHERE id = ?4",
            params![item_name, id],
        )?;

        Ok(changed_rows)
    }

    pub fn get_list_items(&self, list_type: &str) -> DatabaseResult<Vec<ListItem>> {
        // type inferrence doesn't seem to work in `query_map` below if we apply this lint
        // so disable it to avoid useless warnings
        #![allow(clippy::redundant_closure)]

        let db = self.get_connection()?;
        let mut stmt = db.prepare(
            "SELECT id, list_type, item_name, created_s FROM list_item WHERE list_type = ?1",
        )?;
        let list_item_rows: Vec<_> = stmt
            .query_map(params![list_type], |row| ListItem::try_from(row))?
            .collect::<Result<_, _>>()?;

        Ok(list_item_rows)
    }

    pub fn delete_list_item(&self, id: &str) -> DatabaseResult<bool> {
        let db = self.get_connection()?;

        let changed_rows = db.execute("DELETE FROM list_item WHERE id = ?1", params![id])?;

        Ok(changed_rows == 1)
    }

    pub fn rename_list_type(&self, old_name: &str, new_name: &str) -> DatabaseResult<usize> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "UPDATE list_item SET list_type = ?1 WHERE list_type = ?2",
            params![new_name, old_name],
        )?;

        Ok(changed_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn created_utc_test() {
        let list_item = ListItem::new(
            "asd".to_string(),
            "list_type".to_string(),
            "item_name".to_string(),
            1613988164,
        );

        assert_eq!(
            list_item.created_utc(),
            DateTime::parse_from_rfc3339("2021-02-22T10:02:44-00:00").unwrap()
        );
    }
}
