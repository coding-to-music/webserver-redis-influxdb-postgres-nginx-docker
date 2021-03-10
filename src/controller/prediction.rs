use crate::AppError;
use chrono::Utc;
use std::{convert::TryFrom, sync::Arc};
use uuid::Uuid;
use webserver_contracts::{prediction::*, Error as JsonRpcError};
use webserver_database::{Database, Prediction as DbPrediction};

pub struct PredictionController {
    db: Arc<Database<DbPrediction>>,
}

impl PredictionController {
    pub fn new(db: Arc<Database<DbPrediction>>) -> Self {
        Self { db }
    }

    pub async fn add(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<AddPredictionResult, AppError> {
        let params = AddPredictionParams::try_from(request)?;
        let created_s = Utc::now().timestamp() ;

        let id = Uuid::new_v4();
        let db_prediction = DbPrediction::new(id, params.prediction, params.passphrase, created_s);

        let result = self.db.insert_prediction(
            db_prediction.id,
            &db_prediction.text,
            &db_prediction.passphrase,
        )?;

        Ok(if result {
            AddPredictionResult::new(result, Some(id))
        } else {
            AddPredictionResult::new(result, None)
        })
    }

    pub async fn delete(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<DeletePredictionResult, AppError> {
        let params = DeletePredictionParams::try_from(request)?;

        let deleted = self.db.delete_prediction(params.id)?;

        Ok(DeletePredictionResult::new(deleted == 1))
    }

    pub async fn get(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<GetPredictionResult, AppError> {
        let params = GetPredictionParams::try_from(request)?;
        let db_prediction = self.db.get_prediction(params.id)?;

        Ok(GetPredictionResult::new(db_prediction.map(db_to_contract)))
    }
}

impl From<AddPredictionParamsInvalid> for AppError {
    fn from(error: AddPredictionParamsInvalid) -> Self {
        match error {
            AddPredictionParamsInvalid::InvalidFormat(e) => JsonRpcError::invalid_format(e).into(),
            AddPredictionParamsInvalid::EmptyText => JsonRpcError::invalid_params()
                .with_data("text cannot be empty")
                .into(),
            AddPredictionParamsInvalid::PassphraseTooShort => JsonRpcError::invalid_params()
                .with_message("passphrase too short")
                .into(),
        }
    }
}

impl From<DeletePredictionParamsInvalid> for AppError {
    fn from(error: DeletePredictionParamsInvalid) -> Self {
        match error {
            DeletePredictionParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}

impl From<GetPredictionParamsInvalid> for AppError {
    fn from(error: GetPredictionParamsInvalid) -> Self {
        match error {
            GetPredictionParamsInvalid::InvalidFormat(e) => JsonRpcError::invalid_format(e).into(),
        }
    }
}

fn db_to_contract(db_prediction: DbPrediction) -> Prediction {
    Prediction::new(
        db_prediction.id,
        db_prediction.text,
        db_prediction.created_s,
    )
}
