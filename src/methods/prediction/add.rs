use super::*;
use crate::methods;
use std::convert::TryFrom;

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

#[derive(serde::Deserialize)]
pub struct AddPredictionParamsBuilder {
    prediction: String,
    user: methods::User,
}

impl AddPredictionParamsBuilder {
    pub fn build(self) -> Result<AddPredictionParams, AddPredictionParamsInvalid> {
        if self.prediction.is_empty() {
            Err(AddPredictionParamsInvalid::EmptyText)
        } else if self.prediction.len() > 50 {
            Err(AddPredictionParamsInvalid::TextTooLong)
        } else {
            Ok(AddPredictionParams {
                prediction: self.prediction,
                user: self.user,
            })
        }
    }
}

impl TryFrom<serde_json::Value> for AddPredictionParams {
    type Error = AddPredictionParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: AddPredictionParamsBuilder = serde_json::from_value(value)
            .map_err(|e| AddPredictionParamsInvalid::InvalidFormat(e))?;

        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for AddPredictionParams {
    type Error = AddPredictionParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

pub enum AddPredictionParamsInvalid {
    InvalidFormat(serde_json::Error),
    EmptyText,
    TextTooLong,
}

impl From<AddPredictionParamsInvalid> for crate::methods::Error {
    fn from(error: AddPredictionParamsInvalid) -> Self {
        match error {
            AddPredictionParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!("{}", e))
            }
            AddPredictionParamsInvalid::EmptyText => {
                Self::invalid_params().with_data("prediction can't be empty")
            }
            AddPredictionParamsInvalid::TextTooLong => {
                Self::invalid_params().with_data("prediction must not be longer than 50 characters")
            }
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
