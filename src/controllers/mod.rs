pub(crate) use geofences::GeofencesController;
pub(crate) use geofences::GetGeofenceParams;
pub(crate) use geofences::GetNearbyGeofencesParams;
pub(crate) use gis::GisController;
pub(crate) use gis::HaversineParams;
pub(crate) use positions::{
    GetDrivenDistanceParams, GetPositionHistoryParams, PositionsController,
};
use schemars::{schema_for, JsonSchema};
use serde::Serialize;
pub(crate) use sleep::{SleepController, SleepParams};

mod geofences;
mod gis;
mod positions;
mod sleep;

pub trait Schema {
    fn schema() -> serde_json::Value;
}

impl<T: Serialize + JsonSchema> Schema for T {
    fn schema() -> serde_json::Value {
        let schema = schema_for!(T);
        serde_json::to_value(&schema).unwrap()
    }
}
