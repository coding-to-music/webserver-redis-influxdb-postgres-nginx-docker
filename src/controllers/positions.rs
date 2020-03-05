use crate::app::Error;
use chrono::DateTime;
use chrono::Utc;
use core::convert::{TryFrom, TryInto};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::time::{self, SystemTime};

pub struct PositionsController {
    client: reqwest::Client,
}

impl PositionsController {
    pub fn new() -> Self {
        info!("Creating new Positions controller...");
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn get_driven_distance<T: TryInto<GetDrivenDistanceParams, Error = Error>>(
        &self,
        params: T,
    ) -> Result<GetDrivenDistanceResponse, Error> {
        let params = params.try_into()?;
        params.log();

        let request_url = format!(
            "{}/v1/positions/{}/history/distance?fromDate={}&toDate={}",
            Self::url(),
            params.vehicle(),
            epoch_to_datetime(params.start_time() as i64).timestamp(),
            epoch_to_datetime(params.end_time()? as i64).timestamp()
        );
        trace!("GET {}", request_url);

        let iot_response = self
            .client
            .get(&request_url)
            .header("x-functions-key", Self::key())
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                Error::internal_error()
            })?
            .json::<IotDrivenDistanceResponse>()
            .await
            .map_err(|e| {
                error!("{}", e);
                Error::internal_error()
            })?;

        Ok(GetDrivenDistanceResponse::from(iot_response))
    }

    pub(crate) async fn get_position_history<
        T: TryInto<GetPositionHistoryParams, Error = Error>,
    >(
        &self,
        params: T,
    ) -> Result<GetPositionHistoryResponse, Error> {
        let params = params.try_into()?;
        params.log();

        let request_url = format!(
            "{}/v1/positions/{}/history?fromDate={}&toDate={}",
            Self::url(),
            params.vehicle(),
            epoch_to_datetime(params.start_time() as i64).timestamp(),
            epoch_to_datetime(params.end_time()? as i64).timestamp()
        );

        let iot_response = self
            .client
            .get(&request_url)
            .header("x-functions-key", Self::key())
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                Error::internal_error()
            })?
            .json::<IotPositionHistoryResponse>()
            .await
            .map_err(|e| {
                error!("{}", e);
                Error::internal_error()
            })?;

        Ok(GetPositionHistoryResponse::from(iot_response))
    }

    fn url() -> String {
        crate::get_env_var("IOT_POSITIONS_API_URL")
    }

    fn key() -> String {
        crate::get_env_var("IOT_POSITIONS_API_KEY")
    }
}

fn epoch_to_datetime(timestamp: i64) -> DateTime<Utc> {
    let naive = chrono::NaiveDateTime::from_timestamp(timestamp, 0);

    let utc = DateTime::from_utc(naive, Utc);
    utc
}

#[derive(Deserialize)]
pub struct GetDrivenDistanceParams {
    vehicle: String,
    start_time: u128,
    end_time: Option<u128>,
}

impl GetDrivenDistanceParams {
    fn vehicle(&self) -> &str {
        &self.vehicle
    }

    fn start_time(&self) -> u128 {
        self.start_time
    }

    fn end_time(&self) -> Result<u128, Error> {
        self.end_time
            .ok_or_else(|| Error::internal_error().with_message("end time is not defined"))
    }

    fn log(&self) {
        trace!(
            "Get driven distance for {} between {} and {}",
            self.vehicle(),
            self.start_time(),
            self.end_time().unwrap_or(0)
        );
    }
}

impl TryFrom<Value> for GetDrivenDistanceParams {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut params =
            serde_json::from_value::<Self>(value).map_err(|_| Error::invalid_params())?;

        if params.end_time.is_none() {
            params.end_time = Some(
                SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .map_err(|_| Error::internal_error())?
                    .as_millis(),
            );
        }

        if params.vehicle().is_empty() {
            return Err(Error::invalid_params().with_message("vehicle cannot be an empty string"));
        }

        if time::Duration::from_secs((params.end_time()? - params.start_time()) as u64)
            > time::Duration::from_secs(10 * 3_600)
        {
            return Err(
                Error::invalid_params().with_message("duration cannot be larger than 10 hours")
            );
        }

        Ok(params)
    }
}

#[derive(Deserialize)]
pub struct GetPositionHistoryParams {
    vehicle: String,
    start_time: u128,
    end_time: Option<u128>,
}

impl GetPositionHistoryParams {
    fn vehicle(&self) -> &str {
        &self.vehicle
    }

    fn start_time(&self) -> u128 {
        self.start_time
    }

    fn end_time(&self) -> Result<u128, Error> {
        self.end_time
            .ok_or_else(|| Error::internal_error().with_message("end time is not defined"))
    }

    fn log(&self) {
        trace!(
            "Get driven distance for {} between {} and {}",
            self.vehicle(),
            self.start_time(),
            self.end_time().unwrap_or(0)
        );
    }
}

impl TryFrom<Value> for GetPositionHistoryParams {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut params =
            serde_json::from_value::<Self>(value).map_err(|_| Error::invalid_params())?;

        if params.end_time.is_none() {
            params.end_time = Some(
                SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .map_err(|_| Error::internal_error())?
                    .as_millis(),
            );
        }

        if params.vehicle().is_empty() {
            return Err(Error::invalid_params().with_message("vehicle cannot be an empty string"));
        }

        if time::Duration::from_secs((params.end_time()? - params.start_time()) as u64)
            > time::Duration::from_secs(10 * 3_600)
        {
            return Err(
                Error::invalid_params().with_message("duration cannot be larger than 10 hours")
            );
        }

        Ok(params)
    }
}

#[derive(Serialize)]
pub struct GetDrivenDistanceResponse {
    meters: f64,
}

impl From<IotDrivenDistanceResponse> for GetDrivenDistanceResponse {
    fn from(iot_response: IotDrivenDistanceResponse) -> Self {
        Self {
            meters: iot_response.data.meters,
        }
    }
}

#[derive(Serialize)]
pub struct GetPositionHistoryResponse {
    positions: Vec<Position>,
}

impl From<IotPositionHistoryResponse> for GetPositionHistoryResponse {
    fn from(iot_response: IotPositionHistoryResponse) -> Self {
        Self {
            positions: iot_response
                .data
                .into_iter()
                .map(|iot_position| Position::from(iot_position))
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct Position {
    vehicle: String,
    latitude: f32,
    longitude: f32,
    speed: f32,
    bearing: f32,
    timestamp: u64,
}

impl From<IotPositionHistoryResponseData> for Position {
    fn from(response_data: IotPositionHistoryResponseData) -> Self {
        use std::str::FromStr;
        Self {
            vehicle: response_data.vehicle_identity,
            latitude: response_data.latitude,
            longitude: response_data.longitude,
            speed: response_data.speed,
            bearing: response_data.bearing,
            timestamp: chrono::DateTime::<Utc>::from_str(&response_data.timestamp)
                .unwrap()
                .timestamp() as u64,
        }
    }
}

#[derive(Deserialize)]
struct IotDrivenDistanceResponse {
    #[serde(rename = "data")]
    data: IotDrivenDistanceResponseData,
    #[serde(rename = "type")]
    r#_type: String,
}

#[derive(Deserialize)]
struct IotDrivenDistanceResponseData {
    #[serde(rename = "meters")]
    meters: f64,
    #[serde(rename = "startTime")]
    _start_time: String,
    #[serde(rename = "endTime")]
    _end_time: String,
    #[serde(rename = "seconds")]
    _seconds: f32,
}

#[derive(Deserialize)]
struct IotPositionHistoryResponse {
    #[serde(rename = "data")]
    data: Vec<IotPositionHistoryResponseData>,
    #[serde(rename = "type")]
    r#_type: String,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct IotPositionHistoryResponseData {
    vehicle_identity: String,
    latitude: f32,
    longitude: f32,
    altitude: f32,
    bearing: f32,
    speed: f32,
    timestamp: String,
}