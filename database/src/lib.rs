#![allow(unused)]

use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use std::time;
use std::{convert::From, fmt::Debug};
use std::{fmt::Display, marker::PhantomData};

#[macro_use]
extern crate log;

mod list;
mod server;
mod shape;

pub use list::*;
pub use server::*;
pub use shape::*;

pub type DatabaseResult<T> = Result<T, DatabaseError>;

pub struct Database<T> {
    path: String,
    pool: sqlx::PgPool,
    _phantom: PhantomData<T>,
}

impl<T> Database<T> {
    pub async fn new(path: String) -> Result<Self, DatabaseError> {
        Ok(Self {
            pool: sqlx::PgPool::connect(&path).await?,
            path,
            _phantom: PhantomData,
        })
    }

    async fn get_connection(&self) -> Result<PoolConnection<Postgres>, DatabaseError> {
        trace!("connecting to database at '{}'", self.path);
        let timer = time::Instant::now();
        let connection = self.pool.acquire().await?;
        trace!(
            "successfully connected to database at '{}' in {:?}",
            self.path,
            timer.elapsed()
        );
        Ok(connection)
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    SqlxError(sqlx::Error),
    NotAuthorized,
}

impl From<sqlx::Error> for DatabaseError {
    fn from(sqlx_err: sqlx::Error) -> Self {
        DatabaseError::SqlxError(sqlx_err)
    }
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            DatabaseError::SqlxError(e) => format!("sqlx error: '{}'", e.to_string()),
            DatabaseError::NotAuthorized => "not authorized".to_string(),
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
    pub(crate) fn from_changed_rows(changed_rows: u64) -> Self {
        if changed_rows == 1 {
            Self::Inserted
        } else if changed_rows == 0 {
            Self::AlreadyExists
        } else {
            panic!("insertion resulted in {} changed rows", changed_rows);
        }
    }
}
