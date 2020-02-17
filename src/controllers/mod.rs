pub(crate) use geofences::GeofencesController;
pub(crate) use gis::GisController;
pub(crate) use sleep::SleepController;

mod geofences;
mod gis;
mod sleep;

trait Schema {
    fn schema() -> serde_json::Value;
}
