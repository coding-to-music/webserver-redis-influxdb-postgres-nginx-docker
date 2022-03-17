pub mod get_departures;

#[derive(serde::Serialize, Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct Departure {
    pub time: String,
    pub direction: String,
}

impl Departure {
    pub fn new(time: String, direction: String) -> Self {
        Self { time, direction }
    }
}
