use crate::db;
use chrono::prelude::*;
use std::{convert::TryInto, sync::Arc};

mod add;
mod delete;

pub struct PredictionController {
    prediction_db: Arc<db::Database<db::Prediction>>,
    user_db: Arc<db::Database<db::User>>,
}

impl PredictionController {
    pub fn new(
        prediction_db: Arc<db::Database<db::Prediction>>,
        user_db: Arc<db::Database<db::User>>,
    ) -> Self {
        Self {
            prediction_db,
            user_db,
        }
    }

    pub async fn add<
        T: TryInto<add::AddPredictionParams, Error = add::AddPredictionParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<add::AddPredictionResult, super::Error> {
        let params: add::AddPredictionParams = params.try_into()?;

        if self.user_db.validate_user(params.user()) {
            let prediction_row = db::Prediction::new(
                params.user().username().to_owned(),
                params.prediction().to_owned(),
                Utc::now().timestamp() as u32,
            );

            let result = self.prediction_db.insert_prediction(prediction_row)?;

            Ok(add::AddPredictionResult::new(result))
        } else {
            Err(crate::Error::invalid_params().with_data("invalid user"))
        }
    }

    pub async fn delete<
        T: TryInto<delete::DeletePredictionParams, Error = delete::DeletePredictionParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<delete::DeletePredictionResult, crate::Error> {
        let params: delete::DeletePredictionParams = params.try_into()?;

        let success = self.prediction_db.delete_prediction(params.id())?;

        Ok(delete::DeletePredictionResult::new(success))
    }
}
