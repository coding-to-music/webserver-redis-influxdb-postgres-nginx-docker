use crate::AppError;
use chrono::Utc;
use db;
use std::{convert::TryFrom, sync::Arc};
use webserver_contracts::{prediction::*, user::User, Error as JsonRpcError};

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
    ) -> Result<AddPredictionResult, AppError> {
        let params = AddPredictionParams::try_from(request)?;

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
            Err(AppError::from(
                webserver_contracts::Error::invalid_username_or_password(),
            ))
        }
    }

    pub async fn delete(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<DeletePredictionResult, AppError> {
        let params = DeletePredictionParams::try_from(request)?;

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
                _ => Err(AppError::from(JsonRpcError::internal_error().with_data(
                    "can't delete predictions that don't exist, or belong to another user",
                ))),
            }
        } else {
            Err(AppError::from(JsonRpcError::invalid_username_or_password()))
        }
    }

    pub async fn search(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<SearchPredictionsResult, AppError> {
        let params = SearchPredictionsParams::try_from(request)?;

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
                        Prediction::new(
                            if user.username() == params.username() {
                                db_pred.id()
                            } else {
                                None
                            },
                            db_pred.text().to_owned(),
                            db_pred.timestamp_s(),
                        )
                    })
                    .collect(),
            )),
            // An invalid user was provided, return an error
            (Some(_user), false) => {
                Err(AppError::from(JsonRpcError::invalid_username_or_password()))
            }
            // No user was provided, don't show ids
            (None, _) => Ok(SearchPredictionsResult::new(
                predictions
                    .into_iter()
                    .map(|row| Prediction::new(None, row.text().to_owned(), row.timestamp_s()))
                    .collect(),
            )),
        }
    }

    /// Finds a user in the database and validates that the given password is correct
    fn get_and_validate_user(&self, user: &User) -> Result<bool, db::DatabaseError> {
        let valid = self
            .user_db
            .get_user(user.username())?
            .map(|u| {
                let encrypted_password = crate::encrypt(user.password().as_bytes(), u.salt());
                u.validate_password(&encrypted_password)
            })
            .unwrap_or(false);

        Ok(valid)
    }
}

impl From<AddPredictionParamsInvalid> for AppError {
    fn from(error: AddPredictionParamsInvalid) -> Self {
        match error {
            AddPredictionParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
            AddPredictionParamsInvalid::EmptyText => {
                AppError::from(JsonRpcError::invalid_params().with_data("text cannot be empty"))
            }
            AddPredictionParamsInvalid::TextTooLong => {
                AppError::from(JsonRpcError::invalid_params().with_data("text is too long"))
            }
        }
    }
}

impl From<DeletePredictionParamsInvalid> for AppError {
    fn from(error: DeletePredictionParamsInvalid) -> Self {
        match error {
            DeletePredictionParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
            DeletePredictionParamsInvalid::InvalidId => {
                AppError::from(JsonRpcError::invalid_params().with_data("invalid id"))
            }
        }
    }
}

impl From<SearchPredictionsParamsInvalid> for AppError {
    fn from(error: SearchPredictionsParamsInvalid) -> Self {
        match error {
            SearchPredictionsParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
            SearchPredictionsParamsInvalid::EmptyUsername => {
                AppError::from(JsonRpcError::invalid_params().with_data("username can't be empty"))
            }
        }
    }
}
