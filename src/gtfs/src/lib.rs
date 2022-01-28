#[macro_use]
extern crate log;
#[macro_use]
extern crate rouille;

pub mod consts {
    pub const AGENCY_REDIS_KEY: &'static str = "agency";
    pub const CALENDAR_REDIS_KEY: &'static str = "calendar";
    pub const STOP_REDIS_KEY: &'static str = "stop";
    pub const ROUTE_REDIS_KEY: &'static str = "route";
}

mod model;
mod modes;

pub use modes::Populate;
pub use modes::Serve;
