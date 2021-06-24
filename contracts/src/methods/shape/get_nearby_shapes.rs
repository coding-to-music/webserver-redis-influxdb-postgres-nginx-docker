use super::Shape;
use crate::JsonRpcRequest;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::Display,
};

const MIN_COUNT: usize = 1;
const MAX_COUNT: usize = 100;
const DEFAULT_COUNT: usize = 10;
const MIN_DISTANCE_M: u32 = 1;
const MAX_DISTANCE_M: u32 = 500;
const DEFAULT_DISTANCE_M: u32 = 100;
const MIN_LATITUDE: f64 = -90.0;
const MAX_LATITUDE: f64 = 90.0;
const MIN_LONGITUDE: f64 = -180.0;
const MAX_LONGITUDE: f64 = 180.0;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ParamsBuilder")]
#[non_exhaustive]
pub struct Params {
    pub lat: f64,
    pub lon: f64,
    pub count: usize,
    pub distance_m: u32,
}

impl Params {
    /// ## Error
    /// * If `lat` is an invalid latitude.
    /// * If `lon` is an invalid longitude.
    /// * If `count` is outside the range (1..=100).
    /// * If `distance_m` is outside the range (1..=500).
    pub fn new(
        lat: f64,
        lon: f64,
        count: Option<usize>,
        distance_m: Option<u32>,
    ) -> Result<Self, InvalidParams> {
        if !(MIN_LATITUDE..=MAX_LATITUDE).contains(&lat) {
            return Err(InvalidParams::InvalidLatitude);
        }

        if !(MIN_LONGITUDE..=MAX_LONGITUDE).contains(&lon) {
            return Err(InvalidParams::InvalidLongitude);
        }

        let count = match count {
            Some(count) if (MIN_COUNT..=MAX_COUNT).contains(&count) => count,
            None => DEFAULT_COUNT,
            Some(_invalid) => return Err(InvalidParams::InvalidCount),
        };

        let distance_m = match distance_m {
            Some(distance_m) if (MIN_DISTANCE_M..=MAX_DISTANCE_M).contains(&distance_m) => {
                distance_m
            }
            None => DEFAULT_DISTANCE_M,
            Some(_invalid) => return Err(InvalidParams::InvalidDistance),
        };

        Ok(Self {
            lat,
            lon,
            count,
            distance_m,
        })
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
        Self::new(builder.lat, builder.lon, builder.count, builder.distance_m)
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
                format!(
                    "invalid lat, should be float in [{}, {}]",
                    MIN_LATITUDE, MAX_LATITUDE
                )
            }
            InvalidParams::InvalidLongitude => {
                format!(
                    "invalid lon, should be float in [{}, {}]",
                    MIN_LONGITUDE, MAX_LONGITUDE
                )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_test() {
        let valids = [
            Params::new(59.36206482032117, 17.971068620681763, None, None),
            Params::new(59.36206482032117, -175.971068620681763, Some(50), None),
            Params::new(59.36206482032117, -175.971068620681763, None, Some(500)),
        ];
        for valid in &valids {
            assert!(valid.is_ok(), "{:?}", valid);
        }

        let invalids = [Params::new(-150.0, 17.5, None, None)];

        for invalid in &invalids {
            assert!(invalid.is_err(), "{:?}", invalid);
        }
    }

    #[test]
    fn deser() {
        let json = r#"
        {
            "lon": 17.971068620681763,
            "lat": 59.36206482032117
        }
        "#;
        let params = serde_json::from_str::<Params>(&json);
        assert!(params.is_ok(), "{:?}", params);
    }
}
