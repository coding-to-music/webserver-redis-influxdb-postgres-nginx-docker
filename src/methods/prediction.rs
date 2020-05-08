use super::Database;
use chrono::prelude::*;
use rusqlite::{params, Connection};
use std::convert::TryInto;

pub struct PredictionController {
    db_path: String,
}

impl Database for PredictionController {
    fn get_connection(&self) -> Result<Connection, super::Error> {
        let db = Connection::open(&self.db_path).map_err(|e| {
            error!("Failed to connect to database: {:?}", e);
            super::Error::internal_error()
        })?;

        info!("Connected to db");
        Ok(db)
    }
}

impl PredictionController {
    pub fn new() -> Self {
        info!("Creating new PredictionController");
        let db_path = crate::get_env_var("WEBSERVER_SQLITE_PATH");

        Self { db_path }
    }

    fn insert_prediction(&self, prediction: Prediction) -> Result<bool, super::Error> {
        let db = self.get_connection()?;

        let timestamp = Utc::now().timestamp() as u32;

        let changed_rows = db.execute(
            "INSERT INTO prediction (text, timestamp_s, passphrase) VALUES (?1, ?2, ?3)",
            params![prediction.text, timestamp, prediction.passphrase],
        )?;

        Ok(changed_rows == 1)
    }

    pub async fn add<
        T: TryInto<add::AddPredictionParams, Error = add::AddPredictionParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<add::AddPredictionResult, super::Error> {
        let params = params.try_into()?;

        let result = self.insert_prediction(params.prediction)?;

        Ok(add::AddPredictionResult::new(result))
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Prediction {
    text: String,
    passphrase: String,
}

impl Prediction {
    fn has_strong_passphrase(&self) -> bool {
        self.passphrase.len() > 10
    }
}

mod add {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct AddPredictionParams {
        pub(super) prediction: Prediction,
    }

    impl TryFrom<serde_json::Value> for AddPredictionParams {
        type Error = AddPredictionParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params: AddPredictionParams = serde_json::from_value(value)
                .map_err(|_| AddPredictionParamsInvalid::InvalidFormat)?;

            if !params.prediction.has_strong_passphrase() {
                Err(AddPredictionParamsInvalid::WeakPassphrase)
            } else if params.prediction.text.is_empty() {
                Err(AddPredictionParamsInvalid::EmptyText)
            } else if params.prediction.text.len() > 50 {
                Err(AddPredictionParamsInvalid::TextTooLong)
            } else {
                Ok(params)
            }
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for AddPredictionParams {
        type Error = AddPredictionParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            value.params.try_into()
        }
    }

    pub enum AddPredictionParamsInvalid {
        InvalidFormat,
        WeakPassphrase,
        EmptyText,
        TextTooLong,
    }

    impl From<AddPredictionParamsInvalid> for crate::methods::Error {
        fn from(error: AddPredictionParamsInvalid) -> Self {
            match error {
                AddPredictionParamsInvalid::InvalidFormat => Self::invalid_params(),
                AddPredictionParamsInvalid::WeakPassphrase => Self::invalid_params()
                    .with_data("please use a passphrase with more than 10 characters"),
                AddPredictionParamsInvalid::EmptyText => {
                    Self::invalid_params().with_data("prediction can't be empty")
                }
                AddPredictionParamsInvalid::TextTooLong => Self::invalid_params()
                    .with_data("prediction must not be longer than 50 characters"),
            }
        }
    }

    #[derive(serde::Serialize)]
    pub struct AddPredictionResult {
        inserted: bool,
    }

    impl AddPredictionResult {
        pub fn new(inserted: bool) -> Self {
            Self { inserted }
        }
    }
}
