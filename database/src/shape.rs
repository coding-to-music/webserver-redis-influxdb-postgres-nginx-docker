use crate::{Database, DatabaseResult, InsertionResult};
use chrono::{DateTime, TimeZone, Utc};
use sqlx::{Connection, Executor, FromRow, Postgres, Transaction};
use std::{collections::HashMap, convert::TryFrom};

/// This is a row in the `shape` table.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
#[non_exhaustive]
pub struct Shape {
    pub id: String,
    pub name: Option<String>,
    pub geo: String,
    pub deleted_at_s: Option<i64>,
    pub created_s: i64,
}

impl Shape {
    pub fn new(
        id: String,
        name: Option<String>,
        geo: String,
        deleted_at_s: Option<i64>,
        created_s: i64,
    ) -> Self {
        Self {
            id,
            name,
            geo,
            deleted_at_s,
            created_s,
        }
    }

    pub fn created_utc(&self) -> DateTime<Utc> {
        chrono::Utc.timestamp(self.created_s, 0)
    }
}

/// This is a row in the `shape_tag` table.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
#[non_exhaustive]
pub struct ShapeTag {
    pub id: String,
    pub shape_id: String,
    pub tag_name: String,
    pub tag_value: String,
    pub created_s: i64,
}

impl ShapeTag {
    pub fn new(
        id: String,
        shape_id: String,
        tag_name: String,
        tag_value: String,
        created_s: i64,
    ) -> Self {
        Self {
            id,
            shape_id,
            tag_name,
            tag_value,
            created_s,
        }
    }

    pub fn created_utc(&self) -> DateTime<Utc> {
        chrono::Utc.timestamp(self.created_s, 0)
    }
}

impl Database<Shape> {
    pub async fn insert_shape(
        &self,
        shape: &Shape,
        tags: &[&ShapeTag],
    ) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection().await?;

        let mut transaction = db.begin().await?;
        let changed_shape_rows = insert_shape(&mut transaction, &shape).await?;
        for tag in tags {
            insert_shape_tag(&mut transaction, tag).await?;
        }
        transaction.commit().await?;

        Ok(InsertionResult::from_changed_rows(changed_shape_rows))
    }

    pub async fn get_shape(&self, id: &str) -> DatabaseResult<Option<Shape>> {
        let mut db = self.get_connection().await?;

        let mut query_result =
            sqlx::query_as::<_, Shape>("SELECT id, name, geo, created_s FROM shape WHERE id = $1")
                .bind(id)
                .fetch_all(&mut db)
                .await?;

        if query_result.is_empty() {
            Ok(None)
        } else if query_result.len() > 1 {
            error!(r#"more than 1 shape with id: "{}""#, id);
            Ok(None)
        } else {
            Ok(Some(query_result.remove(0)))
        }
    }

    pub async fn get_all_shapes(&self) -> DatabaseResult<Vec<Shape>> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query_as::<_, Shape>(
            "
            SELECT id, name, geo, deleted_at_s, created_s
            FROM shape
            ",
        )
        .fetch_all(&mut db)
        .await?;

        Ok(query_result)
    }

    pub async fn get_shapes_by_ids(&self, ids: &[&str]) -> DatabaseResult<Vec<Shape>> {
        let mut db = self.get_connection().await?;

        let query_list = ids
            .iter()
            .map(|id| format!(r#""{}""#, id))
            .collect::<Vec<_>>()
            .join(", ");

        let query_result = sqlx::query_as::<_, Shape>(
            "
            SELECT id, 
                name,
                geo,
                created_s
            FROM shape 
            WHERE id IN ($1)",
        )
        .bind(query_list)
        .fetch_all(&mut db)
        .await?;

        Ok(query_result)
    }

    pub async fn delete_shape(&self, id: &str) -> DatabaseResult<bool> {
        let mut db = self.get_connection().await?;
        let mut transaction = db.begin().await?;
        let shape_query_result = sqlx::query("DELETE FROM shape WHERE id = $1")
            .bind(id)
            .execute(&mut transaction)
            .await?;
        let shape_tag_query_result = sqlx::query("DELETE FROM shape_tag WHERE shape_id = $1")
            .bind(id)
            .execute(&mut transaction)
            .await?;
        transaction.commit().await?;

        let changed_rows =
            shape_query_result.rows_affected() + shape_tag_query_result.rows_affected();

        Ok(changed_rows > 1)
    }

    pub async fn get_shapes_with_tags(
        &self,
        tags: &HashMap<String, String>,
    ) -> DatabaseResult<Vec<Shape>> {
        // let db = self.get_connection().await?;

        // let pivot_columns = tags
        //     .iter()
        //     .map(|(name, _value)| {
        //         format!(
        //             "MAX(CASE WHEN tag_name = '{}' THEN tag_value END) AS {}_tag",
        //             name, name
        //         )
        //     })
        //     .collect::<Vec<_>>()
        //     .join(", ");

        // let and_joins = tags
        //     .iter()
        //     .map(|(name, value)| format!("{}_tag = '{}'", name, value))
        //     .collect::<Vec<_>>()
        //     .join(" AND ");

        // let pivot_query = format!(
        //     "
        //     WITH piv AS (
        //         SELECT shape_id,
        //             {}
        //         FROM shape_tag
        //         GROUP BY shape_id)
        //     SELECT shape.id,
        //         shape.name,
        //         shape.geo,
        //         shape.created_s
        //     FROM piv
        //     JOIN shape ON piv.shape_id = shape.id
        //     WHERE {}",
        //     pivot_columns, and_joins
        // );

        // executing_query(&pivot_query);

        // let shapes: Vec<_> = db
        //     .prepare(&pivot_query)?
        //     .query_map(params![], |row| crate::parse_from_row(row))?
        //     .collect::<Result<_, _>>()?;

        // Ok(shapes)
        todo!();
    }

    pub async fn insert_shape_tag(&self, tag: &ShapeTag) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection().await?;
        let mut transaction = db.begin().await?;
        let changed_rows = insert_shape_tag(&mut transaction, &tag).await?;
        transaction.commit().await?;

        Ok(InsertionResult::from_changed_rows(changed_rows))
    }

    pub async fn get_shape_tag_by_id(&self, id: &str) -> DatabaseResult<Option<ShapeTag>> {
        let mut db = self.get_connection().await?;

        let mut query_result = sqlx::query_as::<_, ShapeTag>(
            "
            SELECT id, 
                shape_id, 
                tag_name, 
                tag_value, 
                created_s 
            FROM shape_tag 
            WHERE id = $1",
        )
        .bind(id)
        .fetch_all(&mut db)
        .await?;

        if query_result.is_empty() {
            Ok(None)
        } else if query_result.len() > 1 {
            error!(r#"more than 1 shape tag with id: "{}""#, id);
            Ok(None)
        } else {
            Ok(Some(query_result.swap_remove(0)))
        }
    }

    pub async fn get_tags_for_shape(&self, shape_id: &str) -> DatabaseResult<Vec<ShapeTag>> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query_as::<_, ShapeTag>(
            "
            SELECT id, 
                shape_id, 
                tag_name, 
                tag_value, 
                created_s 
            FROM shape_tag 
            WHERE shape_id = $1",
        )
        .bind(shape_id)
        .fetch_all(&mut db)
        .await?;

        Ok(query_result)
    }

    pub async fn get_tags_for_shapes(&self, shape_ids: &[&str]) -> DatabaseResult<Vec<ShapeTag>> {
        let mut tags = Vec::new();
        for shape_id in shape_ids {
            let mut shape_tags = self.get_tags_for_shape(shape_id).await?;
            tags.append(&mut shape_tags);
        }

        Ok(tags)
    }

    pub async fn get_tags_by_name_and_value(
        &self,
        tag_name: &str,
        tag_value: &str,
    ) -> DatabaseResult<Vec<ShapeTag>> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query_as::<_, ShapeTag>(
            "SELECT id, shape_id, tag_name, tag_value, created_s FROM shape_tag WHERE tag_name = $1 AND tag_value = $2",
        )
        .bind(tag_name)
        .bind(tag_value)
        .fetch_all(&mut db)
        .await?;

        Ok(query_result)
    }

    pub async fn delete_tag(&self, id: &str) -> DatabaseResult<bool> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query("DELETE FROM shape_tag WHERE id = $1")
            .bind(id)
            .execute(&mut db)
            .await?;

        Ok(query_result.rows_affected() == 1)
    }
}

async fn insert_shape<'a>(
    transaction: &mut Transaction<'a, Postgres>,
    shape: &Shape,
) -> DatabaseResult<u64> {
    let query_result =
        sqlx::query("INSERT INTO shape (id, name, geo, created_s) VALUES ($1, $2, $3, $4)")
            .bind(&shape.id)
            .bind(&shape.name)
            .bind(&shape.geo)
            .bind(shape.created_s)
            .execute(transaction)
            .await?;
    Ok(query_result.rows_affected())
}

async fn insert_shape_tag<'a>(
    transaction: &mut Transaction<'a, Postgres>,
    tag: &ShapeTag,
) -> DatabaseResult<u64> {
    let query_result =
        sqlx::query("INSERT INTO shape_tag (id, shape_id, tag_name, tag_value, created_s) VALUES ($1, $2, $3, $4, $5)")
            .bind(&tag.id)
            .bind(&tag.shape_id)
            .bind(&tag.tag_name)
            .bind(&tag.tag_value)
            .bind(&tag.created_s)
            .execute(transaction)
            .await?;
    Ok(query_result.rows_affected())
}
