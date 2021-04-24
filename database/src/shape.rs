use crate::{Database, DatabaseResult, InsertionResult};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Row, ToSql, Transaction};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Shape {
    pub id: String,
    pub name: Option<String>,
    pub geo: String,
    pub created_s: i64,
}

impl Shape {
    pub fn new(id: String, name: Option<String>, geo: String, created_s: i64) -> Self {
        Self {
            id,
            name,
            geo,
            created_s,
        }
    }

    pub fn created_utc(&self) -> DateTime<Utc> {
        chrono::Utc.timestamp(self.created_s, 0)
    }
}

impl<'a> TryFrom<&Row<'a>> for Shape {
    type Error = rusqlite::Error;
    fn try_from(row: &Row<'a>) -> Result<Self, Self::Error> {
        let id: String = row.get(0)?;
        let name: Option<String> = row.get(1)?;
        let geo: String = row.get(2)?;

        let created_s = row.get(3)?;

        Ok(Shape::new(id, name, geo, created_s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl<'a> TryFrom<&Row<'a>> for ShapeTag {
    type Error = rusqlite::Error;
    fn try_from(row: &Row<'a>) -> Result<Self, Self::Error> {
        let id: String = row.get(0)?;
        let shape_id: String = row.get(1)?;
        let tag_name: String = row.get(2)?;
        let tag_value: String = row.get(3)?;
        let created_s = row.get(4)?;

        Ok(ShapeTag::new(id, shape_id, tag_name, tag_value, created_s))
    }
}

impl Database<Shape> {
    pub fn insert_shape(
        &self,
        shape: &Shape,
        tags: &[&ShapeTag],
    ) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection()?;

        let transaction = db.transaction()?;
        let changed_shape_rows = insert_shape(&transaction, &shape)?;
        let _changed_shape_tag_rows = insert_shape_tags(&transaction, &tags)?;
        transaction.commit()?;

        Ok(InsertionResult::from_changed_rows(changed_shape_rows))
    }

    pub fn get_shape(&self, id: &str) -> DatabaseResult<Option<Shape>> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare("SELECT id, name, geo, created_s FROM shape WHERE id = ?1")?;

        let mut shape_rows: Vec<_> = stmt
            .query_map(params![id], |row| crate::parse_from_row(row))?
            .collect::<Result<_, _>>()?;

        if shape_rows.is_empty() {
            Ok(None)
        } else if shape_rows.len() > 1 {
            error!(r#"more than 1 shape with id: "{}""#, id);
            Ok(None)
        } else {
            Ok(Some(shape_rows.swap_remove(0)))
        }
    }

    pub fn get_shapes_by_ids(&self, ids: &[&str]) -> DatabaseResult<Vec<Shape>> {
        let db = self.get_connection()?;

        let query_list = ids
            .iter()
            .map(|id| format!(r#""{}""#, id))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "
            SELECT id, 
                name,
                geo,
                created_s
            FROM shape 
            WHERE id IN ({})",
            query_list
        );

        debug!(r#"executing query: '{}'"#, query);

        let mut stmt = db.prepare(&query)?;

        let shape_rows: Vec<Shape> = stmt
            .query_map(params![], |row| crate::parse_from_row(row))?
            .collect::<Result<_, _>>()?;

        Ok(shape_rows)
    }

    pub fn delete_shape(&self, id: &str) -> DatabaseResult<bool> {
        let mut db = self.get_connection()?;
        let transaction = db.transaction()?;
        let changed_rows = transaction.execute("DELETE FROM shape WHERE id = ?1", params![id])?;
        let _changed_rows_tag =
            transaction.execute("DELETE FROM shape_tag WHERE shape_id = ?1", params![id])?;
        transaction.commit()?;

        Ok(changed_rows == 1)
    }

    pub fn get_shapes_with_tags(
        &self,
        tags: &HashMap<String, String>,
    ) -> DatabaseResult<Vec<Shape>> {
        let db = self.get_connection()?;

        let pivot_columns = tags
            .iter()
            .map(|(name, _value)| {
                format!(
                    "MAX(CASE WHEN tag_name = '{}' THEN tag_value END) AS {}_tag",
                    name, name
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let and_joins = tags
            .iter()
            .map(|(name, value)| format!("{}_tag = '{}'", name, value))
            .collect::<Vec<_>>()
            .join(" AND ");

        let pivot_query = format!(
            "
            WITH piv AS (
                SELECT shape_id,
                    {}
                FROM shape_tag
                GROUP BY shape_id)
            SELECT shape.id,
                shape.name,
                shape.geo,
                shape.created_s
            FROM piv
            JOIN shape ON piv.shape_id = shape.id
            WHERE {}",
            pivot_columns, and_joins
        );

        executing_query(&pivot_query);

        let shapes: Vec<_> = db
            .prepare(&pivot_query)?
            .query_map(params![], |row| crate::parse_from_row(row))?
            .collect::<Result<_, _>>()?;

        Ok(shapes)
    }

    pub fn insert_shape_tag(&self, tag: &ShapeTag) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection()?;
        let transaction = db.transaction()?;
        let changed_rows = insert_shape_tags(&transaction, &[tag])?;
        transaction.commit()?;

        Ok(InsertionResult::from_changed_rows(changed_rows))
    }

    pub fn get_shape_tag_by_id(&self, id: &str) -> DatabaseResult<Option<ShapeTag>> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "
            SELECT id, 
                shape_id, 
                tag_name, 
                tag_value, 
                created_s 
            FROM shape_tag 
            WHERE id = ?1",
        )?;

        let mut shape_tag_rows: Vec<_> = stmt
            .query_map(params![id], |row| crate::parse_from_row(row))?
            .collect::<Result<_, _>>()?;

        if shape_tag_rows.is_empty() {
            Ok(None)
        } else if shape_tag_rows.len() > 1 {
            error!(r#"more than 1 shape tag with id: "{}""#, id);
            Ok(None)
        } else {
            Ok(Some(shape_tag_rows.swap_remove(0)))
        }
    }

    pub fn get_tags_for_shape(&self, shape_id: &str) -> DatabaseResult<Vec<ShapeTag>> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "
            SELECT id, 
                shape_id, 
                tag_name, 
                tag_value, 
                created_s 
            FROM shape_tag 
            WHERE shape_id = ?1",
        )?;

        let shape_tag_rows: Vec<_> = stmt
            .query_map(params![shape_id], |row| crate::parse_from_row(row))?
            .collect::<Result<_, _>>()?;

        Ok(shape_tag_rows)
    }

    pub fn get_tags_for_shapes(&self, shape_ids: &[&str]) -> DatabaseResult<Vec<ShapeTag>> {
        let db = self.get_connection()?;

        let query_list = shape_ids
            .iter()
            .map(|id| format!(r#""{}""#, id))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "
        SELECT id, 
            shape_id, 
            tag_name, 
            tag_value, 
            created_s 
        FROM shape_tag 
        WHERE shape_id IN ({})",
            query_list
        );

        let shape_tag_rows: Vec<_> = db
            .prepare(&query)?
            .query_map(params![], |row| crate::parse_from_row(row))?
            .collect::<Result<_, _>>()?;

        Ok(shape_tag_rows)
    }

    pub fn get_tags_by_name_and_value(
        &self,
        tag_name: &str,
        tag_value: &str,
    ) -> DatabaseResult<Vec<ShapeTag>> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "
            SELECT id,
                shape_id,
                tag_name,
                tag_value,
                created_s
            FROM shape_tag
            WHERE tag_name = ?1 AND tag_value = ?2
        ",
        )?;

        let shape_tag_rows: Vec<ShapeTag> = stmt
            .query_map(params![tag_name, tag_value], |row| {
                crate::parse_from_row(row)
            })?
            .collect::<Result<_, _>>()?;

        Ok(shape_tag_rows)
    }

    pub fn delete_tag(&self, id: &str) -> DatabaseResult<bool> {
        let db = self.get_connection()?;

        let changed_rows = db.execute("DELETE FROM shape_tag WHERE id = ?1", params![id])?;

        Ok(changed_rows == 1)
    }
}

fn insert_shape(transaction: &Transaction, shape: &Shape) -> DatabaseResult<usize> {
    match transaction.execute(
        "INSERT INTO shape (id, name, geo, created_s) VALUES (?1, ?2, ?3, ?4)",
        params![shape.id, shape.name, shape.geo, shape.created_s],
    ) {
        Ok(changed_rows) => {
            info!("successfully inserted shape with id '{}'", shape.id);
            Ok(changed_rows)
        }
        Err(e) => {
            error!("error inserting shape: '{}'", e);
            Err(e)?
        }
    }
}

fn insert_shape_tags(transaction: &Transaction, tags: &[&ShapeTag]) -> DatabaseResult<usize> {
    let tag_params: Vec<_> = tags
        .iter()
        .flat_map(|t| {
            vec![
                &t.id as &dyn ToSql,
                &t.shape_id as &dyn ToSql,
                &t.tag_name as &dyn ToSql,
                &t.tag_value as &dyn ToSql,
                &t.created_s as &dyn ToSql,
            ]
        })
        .collect();

    let values = (0..tags.len())
        .map(|i| {
            format!(
                "(?{}, ?{}, ?{}, ?{}, ?{})",
                i * 5 + 1,
                i * 5 + 2,
                i * 5 + 3,
                i * 5 + 4,
                i * 5 + 5
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let query = format!(
        "
        INSERT INTO shape_tag (id,
            shape_id,
            tag_name,
            tag_value,
            created_s)
        VALUES {}",
        values
    );
    
    executing_query(&query);

    match transaction.execute(&query, tag_params) {
        Ok(rows) => {
            info!("successfully inserted {}/{} tags", rows, tags.len());
            Ok(rows)
        }
        Err(e) => {
            error!("error inserting shape tags: '{}'", e);
            Err(e)?
        }
    }
}

fn executing_query(query: &str) {
    trace!("executing: {}", query)
}
