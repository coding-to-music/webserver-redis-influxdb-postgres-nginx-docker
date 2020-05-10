use crate::db;
use chrono::prelude::*;
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
}

mod add {
    use super::*;
    use crate::methods;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct AddPredictionParams {
        prediction: String,
        user: methods::User,
    }

    impl AddPredictionParams {
        pub fn prediction(&self) -> &str {
            &self.prediction
        }

        pub fn user(&self) -> &methods::User {
            &self.user
        }
    }

    impl TryFrom<serde_json::Value> for AddPredictionParams {
        type Error = AddPredictionParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params: AddPredictionParams = serde_json::from_value(value)
                .map_err(|_| AddPredictionParamsInvalid::InvalidFormat)?;

            if params.prediction().is_empty() {
                Err(AddPredictionParamsInvalid::EmptyText)
            } else if params.prediction().len() > 50 {
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
        EmptyText,
        TextTooLong,
    }

    impl From<AddPredictionParamsInvalid> for crate::methods::Error {
        fn from(error: AddPredictionParamsInvalid) -> Self {
            match error {
                AddPredictionParamsInvalid::InvalidFormat => Self::invalid_params(),
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
