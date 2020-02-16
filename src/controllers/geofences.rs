use crate::app::Error;
use core::convert::{TryFrom, TryInto};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub struct GeofencesController {
    client: reqwest::Client,
}

impl GeofencesController {
    pub fn new() -> Self {
        info!("Creating new Geofences controller...");
        if std::env::var("IOT_GEOFENCING_API_URL").is_err() {
            panic!(r#"missing required env var "IOT_GEOFENCING_API_URL""#);
        }
        if std::env::var("IOT_GEOFENCING_API_KEY").is_err() {
            panic!(r#"missing required env var "IOT_GEOFENCING_API_KEY""#);
        }

        Self {
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn get_nearby_geofences<
        T: TryInto<GetNearbyGeofencesParams, Error = Error>,
    >(
        &self,
        params: T,
    ) -> Result<GetNearbyGeofencesResult, Error> {
        let params = params.try_into()?;

        let query_params = format!(
            "latitude={}&longitude={}&count={}&distance={}",
            params.lat(),
            params.lon(),
            params.count(),
            params.distance()
        );
        let request_url = format!("{}/v1/geofences/nearby?{}", Self::url(), query_params);
        trace!("GET {}", request_url);
        let geofencing_response = self
            .client
            .get(&request_url)
            .header("x-functions-key", Self::key())
            .send()
            .await;

        let geofences = geofencing_response
            .map_err(|e| {
                error!("{}", e);
                Error::internal_error()
            })?
            .json::<GeofencingApiResponse>()
            .await
            .map_err(|e| {
                error!("{}", e);
                Error::internal_error()
            })?
            .data;

        Ok(GetNearbyGeofencesResult { geofences })
    }

    pub(crate) async fn get_geofence<T: TryInto<GetGeofenceParams, Error = Error>>(
        &self,
        request: T,
    ) -> Result<GetGeofenceResponse, Error> {
        let params = request.try_into()?;

        let request_url = format!("{}/v2/geofences/{}", Self::url(), params.id);
        trace!("GET {}", request_url);
        let geofence = self
            .client
            .get(&request_url)
            .header("x-functions-key", Self::key())
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                Error::internal_error()
            })?
            .json::<Option<Geofence>>()
            .await
            .map_err(|e| {
                error!("{}", e);
                Error::internal_error()
            })?;

        Ok(GetGeofenceResponse { geofence })
    }

    fn url() -> String {
        std::env::var("IOT_GEOFENCING_API_URL")
            .expect(r#"missing required env var "IOT_GEOFENCING_API_URL""#)
    }

    fn key() -> String {
        std::env::var("IOT_GEOFENCING_API_KEY")
            .expect(r#"missing required env var "IOT_GEOFENCING_API_KEY""#)
    }
}

#[derive(Deserialize)]
pub struct GetNearbyGeofencesParams {
    lat: f32,
    lon: f32,
    count: Option<u32>,
    distance: Option<f32>,
}

impl GetNearbyGeofencesParams {
    pub fn lat(&self) -> f32 {
        self.lat
    }
    pub fn lon(&self) -> f32 {
        self.lon
    }
    pub fn count(&self) -> u32 {
        self.count.unwrap_or(10)
    }
    pub fn distance(&self) -> f32 {
        self.distance.unwrap_or(100.0)
    }
}

#[derive(Serialize)]
pub struct GetNearbyGeofencesResult {
    geofences: Vec<Geofence>,
}

impl TryFrom<Value> for GetNearbyGeofencesParams {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let params = serde_json::from_value::<Self>(value).map_err(|_| Error::invalid_params())?;

        if params.count() > 100 {
            Err(Error::invalid_params().with_message("count cannot be higher than 100"))
        } else if params.count() < 1 {
            Err(Error::invalid_params().with_message("count cannot be lower than 1"))
        } else if params.distance() > 100_000. {
            Err(Error::invalid_params().with_message("distance cannot be higher than 100_000"))
        } else if params.distance() < 1. {
            Err(Error::invalid_params().with_message("distance cannot be lower than 1"))
        } else {
            Ok(params)
        }
    }
}

#[derive(Deserialize)]
struct GeofencingApiResponse {
    data: Vec<Geofence>,
}

#[derive(Deserialize, Serialize)]
pub struct Geofence {
    id: String,
    name: String,
    r#type: String,
}

#[derive(Deserialize)]
pub struct GetGeofenceParams {
    id: String,
}

impl TryFrom<Value> for GetGeofenceParams {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let params = serde_json::from_value::<Self>(value).map_err(|e| {
            error!("{}", e);
            Error::invalid_params()
        })?;

        if params.id.is_empty() {
            Err(Error::invalid_params().with_message(r#""id" cannot be empty"#))
        } else {
            Ok(params)
        }
    }
}

#[derive(Serialize)]
pub(crate) struct GetGeofenceResponse {
    geofence: Option<Geofence>,
}
