pub(crate) use geofences::GeofencesController;
pub(crate) use gis::GisController;
pub(crate) use sleep::SleepController;
pub(crate) use positions::PositionsController;

mod geofences;
mod gis;
mod sleep;
mod positions;

trait Schema {
    fn schema() -> serde_json::Value;
}