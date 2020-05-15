use crate::db;
use chrono::prelude::*;
use std::{convert::TryInto, sync::Arc};

mod add;
mod delete;
mod search;

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
                None,
                params.user().username().to_owned(),
                params.prediction().to_owned(),
                Utc::now().timestamp() as u32,
            );

            let result = self.prediction_db.insert_prediction(prediction_row)?;

            Ok(add::AddPredictionResult::new(result))
        } else {
            Err(crate::Error::invalid_params().with_data("invalid username or password"))
        }
    }

    pub async fn delete<
        T: TryInto<delete::DeletePredictionParams, Error = delete::DeletePredictionParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<delete::DeletePredictionResult, crate::Error> {
        let params: delete::DeletePredictionParams = params.try_into()?;

        if self.user_db.validate_user(params.user()) {
            let prediction = self.prediction_db.get_predictions_by_id(params.id())?;
            let same_user = prediction
                .as_ref()
                .map(|pred| pred.username() == params.user().username())
                .unwrap_or(false);

            match (prediction, same_user) {
                (Some(_prediction), true) => {
                    let success = self.prediction_db.delete_prediction(params.id())?;

                    Ok(delete::DeletePredictionResult::new(success))
                }
                _ => Err(crate::Error::invalid_params().with_data(
                    "can't delete predictions that don't exist, or belong to another user",
                )),
            }
        } else {
            Err(crate::Error::invalid_params().with_data("invalid username or password"))
        }
    }

    pub async fn search<
        T: TryInto<search::SearchPredictionsParams, Error = search::SearchPredictionsParamsInvalid>,
    >(
        &self,
        params: T,
    ) -> Result<search::SearchPredictionsResult, crate::Error> {
        let params: search::SearchPredictionsParams = params.try_into()?;

        let valid_user =
            params.user().is_some() && self.user_db.validate_user(params.user().unwrap());

        trace!(
            r#"searching for predictions made by "{}""#,
            params.username()
        );

        let predictions = self
            .prediction_db
            .get_predictions_by_user(params.username())?;

        trace!(
            r#"found {} predictions made by "{}""#,
            predictions.len(),
            params.username()
        );

        match (params.user(), valid_user) {
            (Some(user), true) => Ok(search::SearchPredictionsResult::new(
                predictions
                    .into_iter()
                    .map(|db_pred| {
                        if user.username() == params.username() {
                            Prediction::from_db_with_id(db_pred)
                        } else {
                            Prediction::from_db_without_id(db_pred)
                        }
                    })
                    .collect(),
            )),
            (Some(_user), false) => {
                Err(crate::Error::invalid_params().with_data("invalid username or password"))
            }
            (None, _) => Ok(search::SearchPredictionsResult::new(
                predictions
                    .into_iter()
                    .map(Prediction::from_db_without_id)
                    .collect(),
            )),
        }
    }
}

#[derive(serde::Serialize)]
pub struct Prediction {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    prediction: String,
    timestamp_s: u32,
    timestamp_s_nice: String,
}

impl Prediction {
    pub fn new(id: Option<i64>, prediction: String, timestamp_s: u32) -> Self {
        Self {
            id,
            prediction,
            timestamp_s,
            timestamp_s_nice: Self::timestamp_s_nice(timestamp_s as i64)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        }
    }

    pub fn from_db_with_id(db_prediction: db::Prediction) -> Self {
        Self::new(
            db_prediction.id(),
            db_prediction.text().to_owned(),
            db_prediction.timestamp_s(),
        )
    }

    pub fn from_db_without_id(db_prediction: db::Prediction) -> Self {
        Self::new(
            None,
            db_prediction.text().to_owned(),
            db_prediction.timestamp_s(),
        )
    }

    fn timestamp_s_nice(timestamp_s: i64) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(timestamp_s, 0),
            chrono::Utc,
        )
    }
}
