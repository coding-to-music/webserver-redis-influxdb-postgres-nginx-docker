use rusqlite::{params, Connection};
use std::convert::TryInto;

pub struct BookmarkController {
    db_path: String,
}

impl BookmarkController {
    pub fn new() -> Self {
        info!("Creating new BookmarkController");
        let db_path = crate::get_env_var("WEBSERVER_SQLITE_PATH");

        Self { db_path }
    }

    fn get_connection(&self) -> Result<Connection, super::Error> {
        let db = Connection::open(&self.db_path).map_err(|e| {
            error!("{:?}", e);
            super::Error::internal_error()
        });
        info!("Connected to db");
        db
    }

    fn insert_bookmark(&self, bookmark: Bookmark) {
        let db = self.get_connection().ok();
        if let Some(db) = db {
            db.execute(
                "INSERT INTO bookmark (name, url) VALUES (?1, ?2)",
                params![bookmark.name, bookmark.url],
            )
            .ok();
        }
    }

    fn delete_bookmark(&self, id: u32) {
        let db = self.get_connection().ok();
        if let Some(db) = db {
            db.execute("DELETE FROM bookmark WHERE id = ?1", params![id])
                .ok();
        }
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

        self.insert_bookmark(params.bookmark);

        Ok(add::AddBookmarkResult::new())
    }

    pub async fn delete<
        T: TryInto<delete::DeleteBookmarkParams, Error = delete::DeleteBookmarkParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<delete::DeleteBookmarkResult, super::Error> {
        let params = params.try_into()?;

        self.delete_bookmark(params.id);

        Ok(delete::DeleteBookmarkResult::new())
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

    pub enum AddBookmarkParamsInvalid {
        InvalidFormat,
    }

    impl From<AddBookmarkParamsInvalid> for crate::methods::Error {
        fn from(_: AddBookmarkParamsInvalid) -> Self {
            Self::invalid_params()
        }
    }

    #[derive(serde::Serialize)]
    pub struct AddBookmarkResult {}

    impl AddBookmarkResult {
        pub fn new() -> Self {
            Self {}
        }
    }
}

mod delete {
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct DeleteBookmarkParams {
        pub(super) id: u32,
    }

    impl TryFrom<serde_json::Value> for DeleteBookmarkParams {
        type Error = DeleteBookmarkParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params = serde_json::from_value(value)
                .map_err(|_| DeleteBookmarkParamsInvalid::InvalidFormat)?;

            Ok(params)
        }
    }

    pub enum DeleteBookmarkParamsInvalid {
        InvalidFormat,
    }

    impl From<DeleteBookmarkParamsInvalid> for crate::methods::Error {
        fn from(_: DeleteBookmarkParamsInvalid) -> Self {
            Self::internal_error()
        }
    }

    #[derive(serde::Serialize)]
    pub struct DeleteBookmarkResult {}

    impl DeleteBookmarkResult {
        pub fn new() -> Self {
            Self {}
        }
    }
}
