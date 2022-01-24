use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Agency {
    pub agency_id: String,
    pub agency_name: String,
    pub agency_url: String,
    pub agency_timezone: String,
    pub agency_lang: String,
    pub agency_fare_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Calendar {
    pub service_id: String,
    pub monday: String,
    pub tuesday: String,
    pub wednesday: String,
    pub thursday: String,
    pub friday: String,
    pub saturday: String,
    pub sunday: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CalendarDate {
    pub service_id: String,
    pub date: String,
    pub exception_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Route {
    pub route_id: String,
    pub agency_id: String,
    pub route_short_name: String,
    pub route_long_name: String,
    pub route_type: String,
    pub route_desc: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: String,
    pub stop_lon: String,
    pub location_type: String,
    pub parent_station: String,
    pub platform_code: String,
}
