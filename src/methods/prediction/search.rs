use crate::db;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize)]
pub struct SearchPredictionsParamsBuilder {
    username: String,
    user: Option<crate::methods::User>,
}

impl SearchPredictionsParamsBuilder {
    pub fn build(self) -> Result<SearchPredictionsParams, SearchPredictionsParamsInvalid> {
        Ok(SearchPredictionsParams {
            username: self.username,
            user: self.user,
        })
    }
}

pub struct SearchPredictionsParams {
    username: String,
    user: Option<crate::methods::User>,
}

impl SearchPredictionsParams {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn user(&self) -> Option<&crate::methods::User> {
        match &self.user {
            None => None,
            Some(user) => Some(user),
        }
    }
}

pub enum SearchPredictionsParamsInvalid {
    InvalidFormat(serde_json::Error),
}

impl TryFrom<serde_json::Value> for SearchPredictionsParams {
    type Error = SearchPredictionsParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: SearchPredictionsParamsBuilder = serde_json::from_value(value)
            .map_err(|e| SearchPredictionsParamsInvalid::InvalidFormat(e))?;

        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for SearchPredictionsParams {
    type Error = SearchPredictionsParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

impl From<SearchPredictionsParamsInvalid> for crate::Error {
    fn from(error: SearchPredictionsParamsInvalid) -> Self {
        match error {
            SearchPredictionsParamsInvalid::InvalidFormat(e) => {
                crate::Error::invalid_params().with_data(format!("invalid format: {}", e))
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct SearchPredictionsResult {
    predictions: Vec<Prediction>,
}

impl SearchPredictionsResult {
    pub fn new(predictions: Vec<Prediction>) -> Self {
        Self { predictions }
    }
}

#[derive(serde::Serialize)]
pub struct Prediction {
    id: Option<i64>,
    prediction: String,
    timestamp_s: u32,
}

impl Prediction {
    pub fn new(id: Option<i64>, prediction: String, timestamp_s: u32) -> Self {
        Self {
            id,
            prediction,
            timestamp_s,
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
}
