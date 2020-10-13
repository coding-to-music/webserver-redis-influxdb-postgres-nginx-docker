use crate::{Database, DatabaseError};
use rusqlite::{params, Row};
use std::convert::TryFrom;

impl Database<Prediction> {
    pub fn delete_predictions_by_username(&self, username: &str) -> Result<usize, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "DELETE FROM prediction WHERE username = ?1",
            params![username],
        )?;

        Ok(changed_rows)
    }

    pub fn get_predictions_by_id(&self, id: i64) -> Result<Option<Prediction>, DatabaseError> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "SELECT ROWID, username, text, timestamp_s FROM prediction WHERE ROWID = ?1",
        )?;

        let mut prediction_rows: Vec<_> = stmt
            .query_map(params![id], |row| Ok(Prediction::try_from(row)?))?
            .filter_map(|r| r.ok())
            .collect();

        if prediction_rows.is_empty() {
            Ok(None)
        } else if prediction_rows.len() == 1 {
            Ok(Some(prediction_rows.swap_remove(0)))
        } else {
            error!("more than 1 prediction with id {}", id);
            Ok(None)
        }
    }

    pub fn insert_prediction(&self, prediction: Prediction) -> Result<bool, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO prediction (username, text, timestamp_s) VALUES (?1, ?2, ?3)",
            params![prediction.username, prediction.text, prediction.timestamp_s,],
        )?;

        Ok(changed_rows == 1)
    }

    pub fn delete_prediction(&self, id: i64) -> Result<bool, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute("DELETE FROM prediction WHERE rowid = ?1", params![id])?;

        Ok(changed_rows == 1)
    }

    pub fn get_predictions_by_user(
        &self,
        username: &str,
    ) -> Result<Vec<Prediction>, DatabaseError> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "SELECT rowid, username, text, timestamp_s FROM prediction WHERE username = ?1",
        )?;

        let user_rows: Vec<_> = stmt
            .query_map(params![username], |row| {
                let rowid = row.get(0)?;
                let username = row.get(1)?;
                let text = row.get(2)?;
                let timestamp_s = row.get(3)?;
                Ok(Prediction::new(Some(rowid), username, text, timestamp_s))
            })?
            .filter_map(|b| b.ok())
            .collect();

        Ok(user_rows)
    }
}

pub struct Prediction {
    id: Option<i64>,
    username: String,
    text: String,
    timestamp_s: u32,
}

impl Prediction {
    pub fn new(id: Option<i64>, username: String, text: String, timestamp_s: u32) -> Self {
        Self {
            id,
            username,
            text,
            timestamp_s,
        }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn timestamp_s(&self) -> u32 {
        self.timestamp_s
    }
}

impl<'a> TryFrom<&Row<'a>> for Prediction {
    type Error = rusqlite::Error;
    fn try_from(row: &Row<'a>) -> Result<Self, Self::Error> {
        let rowid = row.get(0)?;
        let username = row.get(1)?;
        let text = row.get(2)?;
        let timestamp_s = row.get(3)?;
        Ok(Prediction::new(rowid, username, text, timestamp_s))
    }
}
