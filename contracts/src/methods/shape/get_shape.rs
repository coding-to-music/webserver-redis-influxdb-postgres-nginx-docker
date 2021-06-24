use super::Shape;
use crate::JsonRpcRequest;
use geojson::Feature;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub id: Uuid,
    pub geojson: Option<bool>,
}

impl Params {
    pub fn new(id: Uuid, geojson: Option<bool>) -> Self {
        Self { id, geojson }
    }
}

impl TryFrom<JsonRpcRequest> for Params {
    type Error = InvalidParams;
    fn try_from(request: JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder =
            serde_json::from_value(request.params).map_err(InvalidParams::InvalidFormat)?;

        builder.try_into()
    }
}

impl TryFrom<ParamsBuilder> for Params {
    type Error = InvalidParams;

    fn try_from(builder: ParamsBuilder) -> Result<Self, Self::Error> {
        Ok(Self::new(builder.id, builder.geojson))
    }
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    id: Uuid,
    geojson: Option<bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub enum MethodResult {
    Shape(Option<Shape>),
    Geojson(Option<Feature>),
}

impl MethodResult {
    pub fn shape(shape: Option<Shape>) -> Self {
        Self::Shape(shape)
    }

    pub fn geojson(geojson: Option<Feature>) -> Self {
        Self::Geojson(geojson)
    }
}
