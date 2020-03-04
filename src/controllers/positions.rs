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

        let request_url = format!(
            "{}/v1/positions/{}/history/distance?fromDate={}&toDate={}",
            Self::url(),
            params.vehicle(),
            params.start_date_time().timestamp(),
            params.end_date_time().timestamp()
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

    fn url() -> String {
        crate::get_env_var("IOT_POSITIONS_API_URL")
    }

    fn key() -> String {
        crate::get_env_var("IOT_POSITIONS_API_KEY")
    }
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

    fn start_date_time(&self) -> DateTime<Utc> {
        let naive = chrono::NaiveDateTime::from_timestamp(self.start_time() as i64, 0);

        let utc = DateTime::from_utc(naive, Utc);
        utc
    }

    fn end_time(&self) -> u128 {
        self.end_time.unwrap()
    }

    fn end_date_time(&self) -> DateTime<Utc> {
        let naive = chrono::NaiveDateTime::from_timestamp(self.end_time() as i64, 0);

        let utc = DateTime::from_utc(naive, Utc);
        utc
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

        if time::Duration::from_millis((params.end_time() - params.start_time()) as u64)
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

#[derive(Deserialize)]
struct IotDrivenDistanceResponse {
    data: IotDrivenDistanceResponseData,
    r#type: String,
}

#[derive(Deserialize)]
struct IotDrivenDistanceResponseData {
    meters: f64,
    #[serde(alias = "startTime")]
    start_time: String,
    #[serde(alias = "endTime")]
    end_time: String,
    seconds: f32,
}
