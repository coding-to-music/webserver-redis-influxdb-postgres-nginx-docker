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

    fn insert_prediction(&self, prediction: PredictionRow) -> Result<bool, super::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO prediction (text, timestamp_s, passphrase) VALUES (?1, ?2, ?3)",
            params![
                prediction.text,
                prediction.timestamp_s,
                prediction.passphrase
            ],
        )?;

        Ok(changed_rows == 1)
    }

    fn get_predictions(&self, passphrase: &str) -> Result<Vec<PredictionRow>, super::Error> {
        let db = self.get_connection()?;

        let mut stmt = db
            .prepare("SELECT text, timestamp_s, passphrase FROM prediction WHERE passphrase = ?1")
            .map_err(|e| {
                error!("{:?}", e);
                super::Error::internal_error()
            })?;

        let prediction_rows: Vec<_> = stmt
            .query_map(params![passphrase], |row| {
                Ok(PredictionRow::new(row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|b| b.ok())
            .collect();

        Ok(prediction_rows)
    }

    fn delete_predictions(&self, passphrase: &str) -> Result<usize, crate::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "DELETE FROM prediction WHERE passphrase = ?1",
            params![passphrase],
        )?;

        Ok(changed_rows)
    }

    pub async fn add<
        T: TryInto<add::AddPredictionParams, Error = add::AddPredictionParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<add::AddPredictionResult, super::Error> {
        let params = params.try_into()?;

        let prediction_row = PredictionRow::new(
            params.prediction().text().to_owned(),
            Utc::now().timestamp() as u32,
            params.prediction().passphrase().to_owned(),
        );

        info!("inserting prediction: {:?}", prediction_row);

        let result = self.insert_prediction(prediction_row)?;

        Ok(add::AddPredictionResult::new(result))
    }

    pub async fn get<
        T: TryInto<get::GetPredictionsParams, Error = get::GetPredictionsParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<get::GetPredictionsResult, crate::Error> {
        let params = params.try_into()?;

        let prediction_rows = self.get_predictions(params.passphrase())?;

        let predictions: Vec<_> = prediction_rows
            .into_iter()
            .map(|row| get::Prediction::from(row))
            .collect();

        Ok(get::GetPredictionsResult::new(predictions))
    }

    pub async fn delete<
        T: TryInto<delete::DeletePredictionsParams, Error = delete::DeletePredictionsParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<delete::DeletePredictionsResult, crate::Error> {
        let params: delete::DeletePredictionsParams = params.try_into()?;

        let deleted_rows = self.delete_predictions(params.passphrase())?;

        Ok(delete::DeletePredictionsResult::new(deleted_rows))
    }
}

#[derive(Debug)]
struct PredictionRow {
    text: String,
    timestamp_s: u32,
    passphrase: String,
}

impl PredictionRow {
    fn new(text: String, timestamp_s: u32, passphrase: String) -> Self {
        Self {
            text,
            timestamp_s,
            passphrase,
        }
    }
}

mod add {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct Prediction {
        text: String,
        passphrase: String,
    }

    impl Prediction {
        pub fn text(&self) -> &str {
            &self.text
        }

        pub fn passphrase(&self) -> &str {
            &self.passphrase
        }

        fn has_strong_passphrase(&self) -> bool {
            self.passphrase.len() > 10
        }
    }

    #[derive(serde::Deserialize)]
    pub struct AddPredictionParams {
        prediction: Prediction,
    }

    impl AddPredictionParams {
        pub fn prediction(&self) -> &Prediction {
            &self.prediction
        }
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

mod get {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Serialize)]
    pub struct Prediction {
        text: String,
        timestamp_s: u32,
    }

    impl Prediction {
        pub fn new(text: String, timestamp_s: u32) -> Self {
            Self { text, timestamp_s }
        }
    }

    impl From<PredictionRow> for Prediction {
        fn from(row: PredictionRow) -> Self {
            Self::new(row.text, row.timestamp_s)
        }
    }

    #[derive(serde::Deserialize)]
    pub struct GetPredictionsParams {
        passphrase: String,
    }

    impl GetPredictionsParams {
        pub fn passphrase(&self) -> &str {
            &self.passphrase
        }
    }

    impl TryFrom<serde_json::Value> for GetPredictionsParams {
        type Error = GetPredictionsParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params: Self = serde_json::from_value(value)
                .map_err(|_| GetPredictionsParamsInvalid::InvalidFormat)?;

            if params.passphrase.is_empty() {
                Err(GetPredictionsParamsInvalid::PassphraseIsEmpty)
            } else {
                Ok(params)
            }
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for GetPredictionsParams {
        type Error = GetPredictionsParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            value.params.try_into()
        }
    }

    pub enum GetPredictionsParamsInvalid {
        InvalidFormat,
        PassphraseIsEmpty,
    }

    impl From<GetPredictionsParamsInvalid> for crate::Error {
        fn from(error: GetPredictionsParamsInvalid) -> Self {
            match error {
                GetPredictionsParamsInvalid::InvalidFormat => Self::invalid_params(),
                GetPredictionsParamsInvalid::PassphraseIsEmpty => {
                    Self::invalid_params().with_data("passphrase can't be empty")
                }
            }
        }
    }

    #[derive(serde::Serialize)]
    pub struct GetPredictionsResult {
        predictions: Vec<Prediction>,
    }

    impl GetPredictionsResult {
        pub fn new(predictions: Vec<Prediction>) -> Self {
            Self { predictions }
        }
    }
}

mod delete {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct DeletePredictionsParams {
        passphrase: String,
    }

    impl DeletePredictionsParams {
        pub fn passphrase(&self) -> &str {
            &self.passphrase
        }
    }

    impl TryFrom<serde_json::Value> for DeletePredictionsParams {
        type Error = DeletePredictionsParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params: Self = serde_json::from_value(value)
                .map_err(|_| DeletePredictionsParamsInvalid::InvalidFormat)?;

            if params.passphrase.is_empty() {
                Err(DeletePredictionsParamsInvalid::PassphraseIsEmpty)
            } else {
                Ok(params)
            }
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for DeletePredictionsParams {
        type Error = DeletePredictionsParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            value.params.try_into()
        }
    }

    pub enum DeletePredictionsParamsInvalid {
        InvalidFormat,
        PassphraseIsEmpty,
    }

    impl From<DeletePredictionsParamsInvalid> for crate::Error {
        fn from(error: DeletePredictionsParamsInvalid) -> Self {
            match error {
                DeletePredictionsParamsInvalid::InvalidFormat => Self::invalid_params(),
                DeletePredictionsParamsInvalid::PassphraseIsEmpty => {
                    Self::invalid_params().with_data("passphrase can't be empty")
                }
            }
        }
    }

    #[derive(serde::Serialize)]
    pub struct DeletePredictionsResult {
        rows: usize,
    }

    impl DeletePredictionsResult {
        pub fn new(rows: usize) -> Self {
            Self { rows }
        }
    }
}
