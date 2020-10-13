use rusqlite::Connection;
use std::convert::From;
use std::marker::PhantomData;
use std::time;

#[macro_use]
extern crate log;

mod prediction;
mod user;

pub use prediction::Prediction;
pub use user::{User, UserRole};

pub struct Database<T> {
    path: String,
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub enum DatabaseError {
    RusqliteError(rusqlite::Error),
    NotAuthorized,
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(rusqlite_error: rusqlite::Error) -> Self {
        DatabaseError::RusqliteError(rusqlite_error)
    }
}

impl<T> Database<T> {
    pub fn new(path: String) -> Self {
        Self {
            path,
            _phantom: PhantomData,
        }
    }

    fn get_connection(&self) -> Result<Connection, DatabaseError> {
        trace!("connecting to database at '{}'", self.path);
        let timer = time::Instant::now();
        let conn = Connection::open(&self.path).map_err(|e| DatabaseError::from(e));
        if conn.is_ok() {
            trace!(
                "successfully connected to database at '{}' in {:?}",
                self.path,
                timer.elapsed()
            );
        }
        conn
    }
}
