use super::Prediction;
use crate::methods;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize)]
pub struct SearchPredictionsParamsBuilder {
    username: String,
    user: Option<methods::User>,
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
