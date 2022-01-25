use serde::{Deserialize, Serialize};

pub(crate) trait Id {
    type Output;

    fn id(&self) -> Self::Output;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Agency {
    pub agency_id: String,
    pub agency_name: String,
    pub agency_url: String,
    pub agency_timezone: String,
    pub agency_lang: String,
    pub agency_fare_url: String,
}

impl Id for Agency {
    type Output = String;

    fn id(&self) -> Self::Output {
        self.agency_id.clone()
    }
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

impl Id for Calendar {
    type Output = String;

    fn id(&self) -> Self::Output {
        self.service_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CalendarDate {
    pub service_id: String,
    pub date: String,
    pub exception_type: String,
}

impl Id for CalendarDate {
    type Output = String;

    fn id(&self) -> Self::Output {
        self.service_id.clone()
    }
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

impl Id for Route {
    type Output = String;

    fn id(&self) -> Self::Output {
        self.route_id.clone()
    }
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

impl Id for Stop {
    type Output = String;

    fn id(&self) -> Self::Output {
        self.stop_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Attribution {
    pub trip_id: String,
    pub organization_name: String,
    pub is_operator: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Shape {
    pub shape_id: String,
    pub shape_pt_lat: String,
    pub shape_pt_lon: String,
    pub shape_pt_sequence: String,
    pub shape_dist_traveled: String,
}

impl Id for Shape {
    type Output = String;

    fn id(&self) -> Self::Output {
        self.shape_id.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StopTime {
    pub trip_id: String,
    pub arrival_time: String,
    pub departure_time: String,
    pub stop_id: String,
    pub stop_sequence: String,
    pub stop_headsign: String,
    pub pickup_type: String,
    pub drop_off_type: String,
    pub shape_dist_traveled: String,
    pub timepoint: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Transfer {
    pub from_stop_id: String,
    pub to_stop_id: String,
    pub transfer_type: String,
    pub min_transfer_time: String,
    pub from_trip_id: String,
    pub to_trip_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Trip {
    pub route_id: String,
    pub service_id: String,
    pub trip_id: String,
    pub trip_headsign: String,
    pub direction_id: String,
    pub shape_id: String,
}
