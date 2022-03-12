use std::convert::TryFrom;

use isahc::{AsyncReadResponseExt, HttpClient};
use model::{
    traffic::{get_departures, Departure},
    JsonRpcRequest,
};
use serde::{Deserialize, Serialize};

use crate::app::{AppResult, ParamsError};

pub struct TrafficController {
    http_client: HttpClient,
    key: String,
}

impl TrafficController {
    pub fn new(http_client: HttpClient, key: String) -> Self {
        Self { http_client, key }
    }

    pub async fn get_departures(
        &self,
        request: JsonRpcRequest,
    ) -> AppResult<get_departures::MethodResult> {
        use get_departures::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let response = self.get_departures_by_id(params.stop_id).await?;

        let departures = response
            .departure
            .into_iter()
            .map(|d| Departure::from(d))
            .collect();

        Ok(MethodResult::new(departures))
    }

    async fn get_departures_by_id(&self, stop_id: String) -> AppResult<ResRobotDepartureResponse> {
        let request = isahc::Request::builder()
            .method("GET")
            .uri(format!("https://api.resrobot.se/v2.1/departureBoard?id={}&format=json&accessId={}&duration=30", stop_id, self.key))
            .body(())?;

        let response: ResRobotDepartureResponse =
            self.http_client.send_async(request).await?.json().await?;

        Ok(response)
    }
}

impl ParamsError for get_departures::InvalidParams {}

#[derive(Serialize, Deserialize)]
struct ResRobotDepartureResponse {
    #[serde(alias = "Departure")]
    departure: Vec<ResRobotDeparture>,
}

#[derive(Serialize, Deserialize)]
struct ResRobotDeparture {
    time: String,
    direction: String,
}

impl From<ResRobotDeparture> for Departure {
    fn from(rrd: ResRobotDeparture) -> Self {
        Departure::new(rrd.time, rrd.direction)
    }
}
