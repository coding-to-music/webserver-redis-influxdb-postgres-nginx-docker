use reqwest::Client;

pub struct GeofencingClient {
    http_client: Client,
    url: String,
    key: String,
}

impl GeofencingClient {
    pub fn new() -> Self {
        let url = crate::get_env_var("IOT_GEOFENCING_API_URL");
        let key = crate::get_env_var("IOT_GEOFENCING_API_KEY");
        Self {
            http_client: Client::new(),
            url,
            key,
        }
    }

    pub async fn get_geofence(
        &self,
        request: GetGeofenceRequest,
    ) -> Result<GetGeofenceResponse, RequestError> {
        let response = self
            .http_client
            .get(&format!("{}/v2/geofences/{}", &self.url, request.id))
            .header("x-functions-key", &self.key)
            .send()
            .await
            .map_err(|_| RequestError::ReqwestError)?
            .json()
            .await
            .map_err(|_| RequestError::ReqwestError)?;

        Ok(response)
    }
}

#[derive(serde::Deserialize)]
pub struct Geofence {
    id: String,
    latitude: f32,
    longitude: f32,
    r#type: String,
}

impl Geofence {
    pub fn new(id: String, latitude: f32, longitude: f32, r#type: String) -> Self {
        Self {
            id,
            latitude,
            longitude,
            r#type,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn latitude(&self) -> f32 {
        self.latitude
    }

    pub fn longitude(&self) -> f32 {
        self.longitude
    }

    pub fn r#type(&self) -> &str {
        &self.r#type
    }
}

pub struct GetGeofenceRequest {
    id: String,
}

impl GetGeofenceRequest {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

pub enum RequestError {
    ReqwestError,
}

#[derive(serde::Deserialize)]
pub struct GetGeofenceResponse {
    geofence: Option<Geofence>,
}

impl GetGeofenceResponse {
    pub fn new(geofence: Option<Geofence>) -> Self {
        Self { geofence }
    }

    pub fn geofence(&self) -> &Option<Geofence> {
        &self.geofence
    }
}
