use super::Database;
use rusqlite::{params, Connection};
use std::convert::TryInto;

pub struct BookmarkController {
    db_path: String,
}

impl Database for BookmarkController {
    fn get_connection(&self) -> Result<Connection, super::Error> {
        let db = Connection::open(&self.db_path).map_err(|e| {
            error!("Failed to connect to database: {:?}", e);
            super::Error::internal_error()
        })?;

        info!("Connected to db");
        Ok(db)
    }
}

impl BookmarkController {
    pub fn new() -> Self {
        info!("Creating new BookmarkController");
        let db_path = crate::get_env_var("WEBSERVER_SQLITE_PATH");

        Self { db_path }
    }

    fn insert_bookmark(&self, bookmark: Bookmark) -> Result<bool, super::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO bookmark (name, url) VALUES (?1, ?2)",
            params![bookmark.name, bookmark.url],
        )?;

        Ok(changed_rows == 1)
    }

    fn delete_bookmark(&self, bookmark: Bookmark) -> Result<bool, super::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "DELETE FROM bookmark WHERE name = ?1 AND url = ?2",
            params![bookmark.name, bookmark.url],
        )?;

        Ok(changed_rows == 1)
    }

    pub async fn search<
        T: TryInto<search::SearchBookmarkParams, Error = search::SearchBookmarkParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<search::SearchBookmarkResult, super::Error> {
        let params: search::SearchBookmarkParams = params.try_into()?;
        let db = self.get_connection()?;

        let mut stmt = db
            .prepare("SELECT name, url FROM bookmark WHERE name LIKE ?1")
            .map_err(|e| {
                error!("{:?}", e);
                super::Error::internal_error()
            })?;

        info!("Executing query {:?}", stmt);

        let bookmarks: Vec<_> = stmt
            .query_map(params![format!("%{}%", params.input())], |row| {
                Ok(Bookmark::new(row.get(0)?, row.get(1)?))
            })?
            .filter_map(|b| b.ok())
            .collect();

        Ok(search::SearchBookmarkResult::new(bookmarks))
    }

    pub async fn add<T: TryInto<add::AddBookmarkParams, Error = add::AddBookmarkParamsInvalid>>(
        &self,
        params: T,
    ) -> Result<add::AddBookmarkResult, super::Error> {
        let params = params.try_into()?;

        let result = self.insert_bookmark(params.bookmark)?;

        Ok(add::AddBookmarkResult::new(result))
    }

    pub async fn delete<
        T: TryInto<delete::DeleteBookmarkParams, Error = delete::DeleteBookmarkParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<delete::DeleteBookmarkResult, super::Error> {
        let params = params.try_into()?;

        let result = self.delete_bookmark(params.bookmark)?;

        Ok(delete::DeleteBookmarkResult::new(result))
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Bookmark {
    name: String,
    url: String,
}

impl Bookmark {
    pub fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

mod search {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct SearchBookmarkParams {
        input: String,
    }

    impl SearchBookmarkParams {
        pub fn input(&self) -> &str {
            &self.input
        }
    }

    impl TryFrom<serde_json::Value> for SearchBookmarkParams {
        type Error = SearchBookmarkParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params: SearchBookmarkParams = serde_json::from_value(value)
                .map_err(|_| SearchBookmarkParamsInvalid::InvalidFormat)?;

            if params.input.is_empty() {
                Err(SearchBookmarkParamsInvalid::InputIsEmpty)
            } else {
                Ok(params)
            }
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for SearchBookmarkParams {
        type Error = SearchBookmarkParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            value.params.try_into()
        }
    }

    pub enum SearchBookmarkParamsInvalid {
        InvalidFormat,
        InputIsEmpty,
    }

    impl From<SearchBookmarkParamsInvalid> for crate::methods::Error {
        fn from(_: SearchBookmarkParamsInvalid) -> Self {
            Self::invalid_params()
        }
    }

    #[derive(serde::Serialize)]
    pub struct SearchBookmarkResult {
        bookmarks: Vec<Bookmark>,
    }

    impl SearchBookmarkResult {
        pub fn new(bookmarks: Vec<Bookmark>) -> Self {
            Self { bookmarks }
        }
    }
}

mod add {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct AddBookmarkParams {
        pub(super) bookmark: Bookmark,
    }

    impl TryFrom<serde_json::Value> for AddBookmarkParams {
        type Error = AddBookmarkParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params = serde_json::from_value(value)
                .map_err(|_| AddBookmarkParamsInvalid::InvalidFormat)?;

            Ok(params)
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for AddBookmarkParams {
        type Error = AddBookmarkParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            value.params.try_into()
        }
    }

    pub enum AddBookmarkParamsInvalid {
        InvalidFormat,
    }

    impl From<AddBookmarkParamsInvalid> for crate::methods::Error {
        fn from(_: AddBookmarkParamsInvalid) -> Self {
            Self::invalid_params()
        }
    }

    #[derive(serde::Serialize)]
    pub struct AddBookmarkResult {
        inserted: bool,
    }

    impl AddBookmarkResult {
        pub fn new(inserted: bool) -> Self {
            Self { inserted }
        }
    }
}

mod delete {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct DeleteBookmarkParams {
        pub(super) bookmark: Bookmark,
    }

    impl TryFrom<serde_json::Value> for DeleteBookmarkParams {
        type Error = DeleteBookmarkParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params = serde_json::from_value(value)
                .map_err(|_| DeleteBookmarkParamsInvalid::InvalidFormat)?;

            Ok(params)
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for DeleteBookmarkParams {
        type Error = DeleteBookmarkParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            value.params.try_into()
        }
    }

    pub enum DeleteBookmarkParamsInvalid {
        InvalidFormat,
    }

    impl From<DeleteBookmarkParamsInvalid> for crate::methods::Error {
        fn from(_: DeleteBookmarkParamsInvalid) -> Self {
            Self::invalid_params()
        }
    }

    #[derive(serde::Serialize)]
    pub struct DeleteBookmarkResult {
        deleted: bool,
    }

    impl DeleteBookmarkResult {
        pub fn new(deleted: bool) -> Self {
            Self { deleted }
        }
    }
}
