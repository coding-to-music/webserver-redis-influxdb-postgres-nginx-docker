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

    pub async fn search<
        T: TryInto<search::SearchBookmarkParams, Error = search::SearchBookmarkParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<search::SearchBookmarkResult, super::Error> {
        let params: search::SearchBookmarkParams = params.try_into()?;
        let db = Connection::open(&self.db_path).map_err(|e| {
            error!("{:?}", e);
            super::Error::internal_error()
        })?;

        info!("Connected to db");

        let mut stmt = db
            .prepare("SELECT url, name, description FROM bookmark WHERE name LIKE ?1")
            .map_err(|e| {
                error!("{:?}", e);
                super::Error::internal_error()
            })?;

        info!("Executing query {:?}", stmt);

        let bookmarks: Vec<_> = stmt
            .query_map(params![params.input()], |row| {
                Ok(Bookmark::new(row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect();

        info!("{:?}", bookmarks);

        unimplemented!()
    }
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct Bookmark {
    url: String,
    name: String,
    description: String,
}

impl Bookmark {
    pub fn new(url: String, name: String, description: String) -> Self {
        Self {
            url,
            name,
            description,
        }
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
}
