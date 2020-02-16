use crate::app::Error;
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

        let distance = GisController::calculate_distance_m(params.from, params.to);
        Ok(HaversineResult { meters: distance })
    }

    fn calculate_distance_m(from: Coord, to: Coord) -> f32 {
        let d_lat = to.lat.to_radians() - from.lat.to_radians();
        let d_lon = to.lon.to_radians() - from.lon.to_radians();
        let a = (d_lat.sin() / 2.0).powi(2)
            + (to.lat.to_radians() * from.lat.to_radians()) * (d_lon / 2.0).powi(2);
        let c = (a.sqrt()).atan2((1.0 - a).sqrt());
        let distance = 2.0 * 6371.0 * c;
        distance * 1000.
    }
}

#[derive(Deserialize)]
pub struct HaversineParams {
    from: Coord,
    to: Coord,
}

impl TryFrom<Value> for HaversineParams {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value::<Self>(value).map_err(|e| {
            error!("{}", e);
            Error::invalid_params()
        })
    }
}

#[derive(Serialize)]
pub struct HaversineResult {
    meters: f32,
}

#[derive(Deserialize)]
pub struct Coord {
    lat: f32,
    lon: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_distance() {
        let expected = 173.6836045003679 * 1000.;
        assert_eq!(
            expected,
            GisController::calculate_distance_m(
                Coord {
                    lat: 58.123,
                    lon: 17.456
                },
                Coord {
                    lat: 57.2,
                    lon: 15.1
                }
            )
        );
    }
}
