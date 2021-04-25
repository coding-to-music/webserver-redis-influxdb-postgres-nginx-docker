use super::Shape;
use std::{convert::TryFrom, error::Error, fmt::Display};

const MIN_COUNT: usize = 1;
const MAX_COUNT: usize = 100;
const DEFAULT_COUNT: usize = 10;
const MIN_DISTANCE_M: u32 = 1;
const MAX_DISTANCE_M: u32 = 500;
const DEFAULT_DISTANCE_M: u32 = 100;

#[derive(Clone, Debug, serde::Serialize)]
#[serde(try_from = "GetNearbyShapesParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub lat: f64,
    pub lon: f64,
    pub count: usize,
    pub distance_m: u32,
}

impl Params {
    pub fn new(
        lat: f64,
        lon: f64,
        count: Option<usize>,
        distance_m: Option<u32>,
    ) -> Result<Self, InvalidParams> {
        if lat < -90.0 || lat > 90.0 {
            Err(InvalidParams::InvalidLatitude)?;
        }

        if lon < -180.0 || lon > 180.0 {
            Err(InvalidParams::InvalidLongitude)?;
        }

        let count = match count {
            Some(count) if count >= MIN_COUNT && count <= MAX_COUNT => count,
            None => DEFAULT_COUNT,
            Some(_invalid) => Err(InvalidParams::InvalidCount)?,
        };

        let distance_m = match distance_m {
            Some(distance_m) if distance_m >= MIN_DISTANCE_M && distance_m <= MAX_DISTANCE_M => {
                distance_m
            }
            None => DEFAULT_DISTANCE_M,
            Some(_invalid) => Err(InvalidParams::InvalidDistance)?,
        };

        Ok(Self {
            lat,
            lon,
            count,
            distance_m,
        })
    }
}

#[derive(Debug)]
pub enum InvalidParams {
    InvalidFormat(serde_json::Error),
    InvalidCount,
    InvalidDistance,
    InvalidLatitude,
    InvalidLongitude,
}

impl Error for InvalidParams {}

impl Display for InvalidParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            InvalidParams::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            InvalidParams::InvalidCount => format!(
                "invalid count, should be integer in [{}, {}]",
                MIN_COUNT, MAX_COUNT
            ),
            InvalidParams::InvalidDistance => format!(
                "invalid distance_m, should be integer in [{}, {}]",
                MIN_DISTANCE_M, MAX_DISTANCE_M
            ),
            InvalidParams::InvalidLatitude => {
                format!("invalid lat, should be float in [-90, 90]")
            }
            InvalidParams::InvalidLongitude => {
                format!("invalid lon, should be float in [-180, 180]")
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Deserialize)]
struct ParamsBuilder {
    lat: f64,
    lon: f64,
    count: Option<usize>,
    distance_m: Option<u32>,
}

impl ParamsBuilder {
    fn build(self) -> Result<Params, InvalidParams> {
        Params::new(self.lat, self.lon, self.count, self.distance_m)
    }
}

impl TryFrom<crate::JsonRpcRequest> for Params {
    type Error = InvalidParams;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: ParamsBuilder = serde_json::from_value(request.params)
            .map_err(InvalidParams::InvalidFormat)?;

        builder.build()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MethodResult {
    pub shape: Vec<Shape>,
}

impl MethodResult {
    pub fn new(shape: Vec<Shape>) -> Self {
        Self { shape }
    }
}