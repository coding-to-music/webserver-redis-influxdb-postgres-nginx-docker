use crate::methods;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize)]
pub struct DeletePredictionParamsBuilder {
    id: i64,
    user: methods::User,
}

impl DeletePredictionParamsBuilder {
    pub fn build(self) -> Result<DeletePredictionParams, DeletePredictionParamsInvalid> {
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
        let builder: DeletePredictionParamsBuilder = serde_json::from_value(value)
            .map_err(|e| DeletePredictionParamsInvalid::InvalidFormat(e))?;

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

impl From<DeletePredictionParamsInvalid> for crate::methods::Error {
    fn from(error: DeletePredictionParamsInvalid) -> Self {
        match error {
            DeletePredictionParamsInvalid::InvalidFormat(e) => {
                Self::invalid_params().with_data(format!("{}", e))
            }
            DeletePredictionParamsInvalid::InvalidId => {
                Self::invalid_params().with_data("id must be greater than 0")
            }
        }
    }
}
