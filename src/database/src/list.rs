use crate::{Database, DatabaseResult, InsertionResult};
use sqlx::{postgres::PgRow, types::time::OffsetDateTime, Row};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
#[non_exhaustive]
pub struct ListItem {
    pub id: String,
    pub list_type: String,
    pub item_name: String,
    pub created: OffsetDateTime,
}

impl ListItem {
    fn new(id: String, list_type: String, item_name: String, created: OffsetDateTime) -> Self {
        Self {
            id,
            list_type,
            item_name,
            created,
        }
    }
}

impl Database<ListItem> {
    pub async fn insert_list_item(
        &self,
        id: &str,
        list_type: &str,
        item_name: &str,
    ) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection().await?;

        let query_result =
            sqlx::query("INSERT INTO list_item (id, list_type, item_name) VALUES ($1, $2, $3)")
                .bind(id)
                .bind(list_type)
                .bind(item_name)
                .execute(&mut db)
                .await?;

        Ok(InsertionResult::from_changed_rows(
            query_result.rows_affected(),
        ))
    }

    pub async fn get_list_item(&self, id: &str) -> DatabaseResult<Option<ListItem>> {
        let mut db = self.get_connection().await?;

        let mut query_result = sqlx::query_as::<_, ListItem>(
            "SELECT id, list_type, item_name, created FROM list_item WHERE id = $1",
        )
        .bind(id)
        .fetch_all(&mut db)
        .await?;

        if query_result.is_empty() {
            Ok(None)
        } else if query_result.len() > 1 {
            error!(r#"more than 1 list item with id: "{}""#, id);
            Ok(None)
        } else {
            Ok(Some(query_result.remove(0)))
        }
    }

    pub async fn get_list_types(&self) -> DatabaseResult<Vec<String>> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query("SELECT DISTINCT list_type FROM list_item")
            .map(|row: PgRow| {
                let list_type: &str = row.get("list_type");
                list_type.to_owned()
            })
            .fetch_all(&mut db)
            .await?;

        Ok(query_result)
    }

    pub async fn update_list_item(&self, id: &str, item_name: &str) -> DatabaseResult<u64> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query("UPDATE list_item SET item_name = $1 WHERE id = $2")
            .bind(item_name)
            .bind(id)
            .execute(&mut db)
            .await?;

        Ok(query_result.rows_affected())
    }

    pub async fn get_list_items(&self, list_type: &str) -> DatabaseResult<Vec<ListItem>> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query_as::<_, ListItem>(
            "SELECT id, list_type, item_name, created FROM list_item WHERE list_type = $1",
        )
        .bind(list_type)
        .fetch_all(&mut db)
        .await?;

        Ok(query_result)
    }

    pub async fn delete_list_item(&self, id: &str) -> DatabaseResult<bool> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query("DELETE FROM list_item WHERE id = $1")
            .bind(id)
            .execute(&mut db)
            .await?;

        Ok(query_result.rows_affected() == 1)
    }

    pub async fn rename_list_type(&self, old_name: &str, new_name: &str) -> DatabaseResult<u64> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query("UPDATE list_item SET list_type = $1 WHERE list_type = $2")
            .bind(new_name)
            .bind(old_name)
            .execute(&mut db)
            .await?;

        Ok(query_result.rows_affected())
    }
}
