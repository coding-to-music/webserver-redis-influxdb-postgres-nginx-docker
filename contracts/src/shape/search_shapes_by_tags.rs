use super::Shape;
use crate::JsonRpcRequest;
use std::{collections::HashMap, convert::TryFrom, error::Error, fmt::Display};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "SearchShapesByTagsParamsBuilder")]
#[non_exhaustive]
pub struct SearchShapesByTagsParams {
    pub or: Vec<HashMap<String, String>>,
}

impl SearchShapesByTagsParams {
    pub fn new(or: Vec<HashMap<String, String>>) -> Result<Self, SearchShapesByTagsParamsInvalid> {
        let o = Self { or };

        o.validate()?;

        Ok(o)
    }

    fn validate(&self) -> Result<(), SearchShapesByTagsParamsInvalid> {
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
struct SearchShapesByTagsParamsBuilder {
    or: Vec<HashMap<String, String>>,
}

impl TryFrom<SearchShapesByTagsParamsBuilder> for SearchShapesByTagsParams {
    type Error = SearchShapesByTagsParamsInvalid;

    fn try_from(builder: SearchShapesByTagsParamsBuilder) -> Result<Self, Self::Error> {
        Self::new(builder.or)
    }
}

#[derive(Debug)]
pub enum SearchShapesByTagsParamsInvalid {
    InvalidFormat(serde_json::Error),
    InvalidName,
    InvalidValue,
}

impl Error for SearchShapesByTagsParamsInvalid {}

impl Display for SearchShapesByTagsParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            SearchShapesByTagsParamsInvalid::InvalidFormat(e) => {
                crate::invalid_params_serde_message(e)
            }
            SearchShapesByTagsParamsInvalid::InvalidName => "invalid tag name".to_string(),
            SearchShapesByTagsParamsInvalid::InvalidValue => "invalid tag value".to_string(),
        };

        write!(f, "{}", output)
    }
}

impl TryFrom<JsonRpcRequest> for SearchShapesByTagsParams {
    type Error = SearchShapesByTagsParamsInvalid;

    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        serde_json::from_value(request.params).map_err(Self::Error::InvalidFormat)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SearchShapesByTagsResult {
    pub shapes: Vec<Shape>,
}

impl SearchShapesByTagsResult {
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self { shapes }
    }
}
