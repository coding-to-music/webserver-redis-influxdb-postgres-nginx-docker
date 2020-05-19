use crate::{db, methods};
use chrono::prelude::*;
use std::convert::{TryFrom, TryInto};

pub use controller::PredictionController;

mod controller;

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
struct AddPredictionParamsBuilder {
    prediction: String,
    user: methods::User,
}

impl AddPredictionParamsBuilder {
    fn build(self) -> Result<AddPredictionParams, AddPredictionParamsInvalid> {
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
        let builder: AddPredictionParamsBuilder =
            serde_json::from_value(value).map_err(AddPredictionParamsInvalid::InvalidFormat)?;

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

impl From<AddPredictionParamsInvalid> for crate::Error {
    fn from(error: AddPredictionParamsInvalid) -> Self {
        match error {
            AddPredictionParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
            AddPredictionParamsInvalid::EmptyText => {
                Self::invalid_params().with_message("prediction can't be empty")
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

#[derive(serde::Deserialize)]
struct DeletePredictionParamsBuilder {
    id: i64,
    user: methods::User,
}

impl DeletePredictionParamsBuilder {
    fn build(self) -> Result<DeletePredictionParams, DeletePredictionParamsInvalid> {
        if self.id <= 0 {
            Err(DeletePredictionParamsInvalid::InvalidId)
        } else {
            Ok(DeletePredictionParams {
                id: self.id,
                user: self.user,
            })
        }
    }
}

pub struct DeletePredictionParams {
    id: i64,
    user: methods::User,
}

impl DeletePredictionParams {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn user(&self) -> &methods::User {
        &self.user
    }
}

pub enum DeletePredictionParamsInvalid {
    InvalidFormat(serde_json::Error),
    InvalidId,
}

impl TryFrom<serde_json::Value> for DeletePredictionParams {
    type Error = DeletePredictionParamsInvalid;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let builder: DeletePredictionParamsBuilder =
            serde_json::from_value(value).map_err(DeletePredictionParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

impl TryFrom<crate::JsonRpcRequest> for DeletePredictionParams {
    type Error = DeletePredictionParamsInvalid;
    fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        value.params.try_into()
    }
}

#[derive(serde::Serialize)]
pub struct DeletePredictionResult {
    success: bool,
}

impl DeletePredictionResult {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}

impl From<DeletePredictionParamsInvalid> for crate::Error {
    fn from(error: DeletePredictionParamsInvalid) -> Self {
        match error {
            DeletePredictionParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
            }
            DeletePredictionParamsInvalid::InvalidId => {
                Self::invalid_params().with_data("id must be greater than 0")
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct SearchPredictionsParamsBuilder {
    username: String,
    user: Option<methods::User>,
}

impl SearchPredictionsParamsBuilder {
    fn build(self) -> Result<SearchPredictionsParams, SearchPredictionsParamsInvalid> {
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
        let builder: SearchPredictionsParamsBuilder =
            serde_json::from_value(value).map_err(SearchPredictionsParamsInvalid::InvalidFormat)?;

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
                Self::invalid_params().with_data(format!(r#"invalid format: "{}""#, e))
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
