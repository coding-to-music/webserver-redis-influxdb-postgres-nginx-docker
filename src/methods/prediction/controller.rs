use super::*;
use crate::{db, methods::User};
use chrono::Utc;
use std::{convert::TryInto, sync::Arc};

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

    pub async fn add(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<AddPredictionResult, crate::Error> {
        let params: AddPredictionParams = request.try_into()?;

        if self.get_and_validate_user(params.user())? {
            let prediction_row = db::Prediction::new(
                None,
                params.user().username().to_owned(),
                params.prediction().to_owned(),
                Utc::now().timestamp() as u32,
            );

            let result = self.prediction_db.insert_prediction(prediction_row)?;

            Ok(AddPredictionResult::new(result))
        } else {
            Err(crate::Error::invalid_username_or_password())
        }
    }

    pub async fn delete(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<DeletePredictionResult, crate::Error> {
        let params: DeletePredictionParams = request.try_into()?;

        if self.get_and_validate_user(params.user())? {
            let prediction = self.prediction_db.get_predictions_by_id(params.id())?;
            let same_user = prediction
                .as_ref()
                .map(|pred| pred.username() == params.user().username())
                .unwrap_or(false);

            match (prediction, same_user) {
                (Some(_prediction), true) => {
                    let success = self.prediction_db.delete_prediction(params.id())?;

                    Ok(DeletePredictionResult::new(success))
                }
                _ => Err(crate::Error::invalid_params().with_data(
                    "can't delete predictions that don't exist, or belong to another user",
                )),
            }
        } else {
            Err(crate::Error::invalid_params().with_data("invalid username or password"))
        }
    }

    pub async fn search(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<SearchPredictionsResult, crate::Error> {
        let params: SearchPredictionsParams = request.try_into()?;

        let valid_user = if let Some(user) = params.user() {
            self.get_and_validate_user(user)?
        } else {
            false
        };

        let predictions = self
            .prediction_db
            .get_predictions_by_user(params.username())?;

        match (params.user(), valid_user) {
            // A valid user was provided, show ids if predictions belong to the given user
            (Some(user), true) => Ok(SearchPredictionsResult::new(
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
            // An invalid user was provided, return an error
            (Some(_user), false) => Err(crate::Error::invalid_username_or_password()),
            // No user was provided, don't show ids
            (None, _) => Ok(SearchPredictionsResult::new(
                predictions
                    .into_iter()
                    .map(Prediction::from_db_without_id)
                    .collect(),
            )),
        }
    }

    fn get_and_validate_user(&self, user: &User) -> Result<bool, db::DatabaseError> {
        let valid = self
            .user_db
            .get_user(user.username())?
            .map(|u| u.validate_password(user.password().as_bytes()))
            .unwrap_or(false);

        Ok(valid)
    }
}
