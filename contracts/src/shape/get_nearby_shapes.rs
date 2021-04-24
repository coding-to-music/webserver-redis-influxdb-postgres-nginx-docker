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
pub struct GetNearbyShapesParams {
    pub lat: f64,
    pub lon: f64,
    pub count: usize,
    pub distance_m: u32,
}

impl GetNearbyShapesParams {
    pub fn new(
        lat: f64,
        lon: f64,
        count: Option<usize>,
        distance_m: Option<u32>,
    ) -> Result<Self, GetNearbyShapesParamsInvalid> {
        if lat < -90.0 || lat > 90.0 {
            Err(GetNearbyShapesParamsInvalid::InvalidLatitude)?;
        }

        if lon < -180.0 || lon > 180.0 {
            Err(GetNearbyShapesParamsInvalid::InvalidLongitude)?;
        }

        let count = match count {
            Some(count) if count >= MIN_COUNT && count <= MAX_COUNT => count,
            None => DEFAULT_COUNT,
            Some(_invalid) => Err(GetNearbyShapesParamsInvalid::InvalidCount)?,
        };

        let distance_m = match distance_m {
            Some(distance_m) if distance_m >= MIN_DISTANCE_M && distance_m <= MAX_DISTANCE_M => {
                distance_m
            }
            None => DEFAULT_DISTANCE_M,
            Some(_invalid) => Err(GetNearbyShapesParamsInvalid::InvalidDistance)?,
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
pub enum GetNearbyShapesParamsInvalid {
    InvalidFormat(serde_json::Error),
    InvalidCount,
    InvalidDistance,
    InvalidLatitude,
    InvalidLongitude,
}

impl Error for GetNearbyShapesParamsInvalid {}

impl Display for GetNearbyShapesParamsInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            GetNearbyShapesParamsInvalid::InvalidFormat(serde_error) => {
                crate::invalid_params_serde_message(&serde_error)
            }
            GetNearbyShapesParamsInvalid::InvalidCount => format!(
                "invalid count, should be integer in [{}, {}]",
                MIN_COUNT, MAX_COUNT
            ),
            GetNearbyShapesParamsInvalid::InvalidDistance => format!(
                "invalid distance_m, should be integer in [{}, {}]",
                MIN_DISTANCE_M, MAX_DISTANCE_M
            ),
            GetNearbyShapesParamsInvalid::InvalidLatitude => {
                format!("invalid lat, should be float in [-90, 90]")
            }
            GetNearbyShapesParamsInvalid::InvalidLongitude => {
                format!("invalid lon, should be float in [-180, 180]")
            }
        };

        write!(f, "{}", output)
    }
}

#[derive(serde::Deserialize)]
struct GetNearbyShapesParamsBuilder {
    lat: f64,
    lon: f64,
    count: Option<usize>,
    distance_m: Option<u32>,
}

impl GetNearbyShapesParamsBuilder {
    fn build(self) -> Result<GetNearbyShapesParams, GetNearbyShapesParamsInvalid> {
        GetNearbyShapesParams::new(self.lat, self.lon, self.count, self.distance_m)
    }
}

impl TryFrom<crate::JsonRpcRequest> for GetNearbyShapesParams {
    type Error = GetNearbyShapesParamsInvalid;
    fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
        let builder: GetNearbyShapesParamsBuilder = serde_json::from_value(request.params)
            .map_err(GetNearbyShapesParamsInvalid::InvalidFormat)?;

        builder.build()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GetNearbyShapesResult {
    pub shape: Vec<Shape>,
}

impl GetNearbyShapesResult {
    pub fn new(shape: Vec<Shape>) -> Self {
        Self { shape }
    }
}
