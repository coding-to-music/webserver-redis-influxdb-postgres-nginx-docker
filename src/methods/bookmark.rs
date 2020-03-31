use rusqlite::Connection;
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
        info!("{}", db.is_autocommit());

        let query = match params.input() {
            Some(input) => format!("SELECT * FROM bookmark WHERE name LIKE '{}'", input),
            None => format!("SELECT * FROM bookmark"),
        };

        match db.execute(query) {
            Ok(count) => info!("Success: {}", count),
            Err(e) => error!("{:?}", e),
        }

        unimplemented!()
    }
}

#[derive(serde::Serialize)]
pub struct Bookmark {
    name: String,
    url: String,
}

mod search {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct SearchBookmarkParams {
        input: Option<String>,
    }

    impl SearchBookmarkParams {
        pub fn input(&self) -> &Option<String> {
            &self.input
        }
    }

    impl TryFrom<serde_json::Value> for SearchBookmarkParams {
        type Error = SearchBookmarkParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params = serde_json::from_value(value)
                .map_err(|_| SearchBookmarkParamsInvalid::InvalidFormat)?;

            Ok(params)
        }
    }

    pub enum SearchBookmarkParamsInvalid {
        InvalidFormat,
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
