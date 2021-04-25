use rusqlite::{Connection, Error, Row};
use std::time;
use std::{
    convert::{From, TryFrom},
    fmt::Debug,
};
use std::{fmt::Display, marker::PhantomData};

#[macro_use]
extern crate log;

mod list;
mod queue;
mod shape;

pub use list::*;
pub use queue::*;
pub use shape::*;

pub type DatabaseResult<T> = Result<T, DatabaseError>;

pub struct Database<T> {
    path: String,
    _phantom: PhantomData<T>,
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
        let conn = Connection::open(&self.path)?;
        trace!(
            "successfully connected to database at '{}' in {:?}",
            self.path,
            timer.elapsed()
        );
        Ok(conn)
    }
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

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            DatabaseError::RusqliteError(e) => e.to_string(),
            DatabaseError::NotAuthorized => {
                format!("not authorized")
            }
        };

        write!(f, "{}", output)
    }
}

impl std::error::Error for DatabaseError {}

#[derive(Clone, Copy, Debug)]
pub enum InsertionResult {
    Inserted,
    AlreadyExists,
}

impl InsertionResult {
    pub(crate) fn from_changed_rows(changed_rows: usize) -> Self {
        if changed_rows == 1 {
            Self::Inserted
        } else if changed_rows == 0 {
            Self::AlreadyExists
        } else {
            panic!("insertion resulted in {} changed rows", changed_rows);
        }
    }
}

fn parse_from_row<'a, T>(row: &'a Row) -> Result<T, Error>
where
    T: TryFrom<&'a Row<'a>, Error = Error> + Debug,
{
    let result = T::try_from(row);

    match result {
        Ok(ok) => {
            trace!("parsed '{:?}' from row", ok);
            Ok(ok)
        }
        Err(err) => {
            error!(
                "failed to parse object of type '{}' from row with error '{:?}'",
                std::any::type_name::<T>(),
                err
            );
            Err(err)
        }
    }
}
