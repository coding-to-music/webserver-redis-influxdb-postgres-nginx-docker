use super::Schema;
use crate::app::Error;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::{TryFrom, TryInto};

pub struct GisController;

impl GisController {
    pub fn new() -> Self {
        Self
    }

    pub fn haversine<T: TryInto<HaversineParams, Error = Error>>(
        &self,
        params: T,
    ) -> Result<HaversineResult, Error> {
        let params = params.try_into()?;

        let distance = params
            .points
            .windows(2)
            .map(|w| GisController::calculate_distance_m(w[0], w[1]))
            .sum();
        Ok(HaversineResult { meters: distance })
    }

    fn calculate_distance_m(from: Coord, to: Coord) -> f32 {
        const R: f32 = 6371.0;
        let d_lat = (to.lat - from.lat).to_radians();
        let d_lon = (to.lon - from.lon).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + from.lat.to_radians().cos() * to.lat.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let distance = R * c;
        distance * 1000.
    }
}

#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct HaversineParams {
    points: Vec<Coord>,
}

impl TryFrom<Value> for HaversineParams {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let params = serde_json::from_value::<Self>(value).map_err(|e| {
            error!("{}", e);
            Error::invalid_params()
                .with_message("failed to deserialize params")
                .with_data(HaversineParams::schema())
        })?;

        if params.points.len() < 2 {
            Err(Error::invalid_params().with_message(r#"at least two coords required"#))
        } else if params
            .points
            .iter()
            .any(|p| p.lat > 90.0 || p.lat < -90.0 || p.lon > 180.0 || p.lon < -180.0)
        {
            Err(Error::invalid_params().with_message(r#"valid range for latitude is [-90, 90], valid range for longitude is [-180, 180]"#))
        } else {
            Ok(params)
        }
    }
}

#[derive(Serialize)]
pub struct HaversineResult {
    meters: f32,
}

#[derive(Deserialize, Debug, JsonSchema, Copy, Clone, Serialize)]
pub struct Coord {
    lat: f32,
    lon: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn haversine_distance() {
        let expected_pair = 173.6836045003679 * 1000.;
        assert!(
            (GisController::calculate_distance_m(
                Coord {
                    lat: 58.123,
                    lon: 17.456
                },
                Coord {
                    lat: 57.2,
                    lon: 15.1
                }
            ) - expected_pair)
                .abs()
                < 0.1f32
        );
    }

    #[test]
    fn params() {
        let _params = HaversineParams::try_from(
            Value::from_str(
                r#"
        {
            "from": {
                "lat": 58.123,
                "lon": 17.456
            },
            "to": {
                "lat": 57.2,
                "lon": 15.1
            }
        }"#,
            )
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn coord() {
        let _coord = serde_json::from_str::<Coord>(
            r#"
        {
            "lat": 58.123,
            "lon": 17.456
        }
        "#,
        )
        .unwrap();
    }
}
