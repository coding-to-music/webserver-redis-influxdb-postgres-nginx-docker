use crate::clients::geofencing::GeofencingClient;
use get_geofence::*;
use std::convert::TryInto;

pub struct GeofencingController {
    client: GeofencingClient,
}

impl GeofencingController {
    pub fn new() -> Self {
        info!("Creating new GeofencingController");
        Self {
            client: GeofencingClient::new(),
        }
    }

    pub async fn get_geofence<T>(&self, params: T) -> Result<GetGeofenceResult, super::Error>
    where
        T: TryInto<GetGeofenceParams, Error = GeofenceParamsInvalid>,
    {
        let params = params.try_into()?;
        let response = self.client.get_geofence(params.into()).await;
        unimplemented!();
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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
}

impl From<crate::clients::geofencing::Geofence> for Geofence {
    fn from(source: crate::clients::geofencing::Geofence) -> Self {
        Geofence::new(source.id, source.latitude, source.longitude, source.r#type)
    }
}

mod get_geofence {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct GetGeofenceParams {
        id: String,
    }

    impl GetGeofenceParams {
        pub fn id(&self) -> &str {
            &self.id
        }
    }

    impl From<GetGeofenceParams> for crate::clients::geofencing::GetGeofenceRequest {
        fn from(source: GetGeofenceParams) -> Self {
            Self::new(source.id().to_owned())
        }
    }

    pub enum GeofenceParamsInvalid {
        InvalidFormat,
        IdIsEmpty,
    }

    impl TryFrom<serde_json::Value> for GetGeofenceParams {
        type Error = GeofenceParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params: GetGeofenceParams =
                serde_json::from_value(value).map_err(|_| GeofenceParamsInvalid::InvalidFormat)?;

            if params.id.is_empty() {
                Err(GeofenceParamsInvalid::IdIsEmpty)
            } else {
                Ok(params)
            }
        }
    }

    impl From<GeofenceParamsInvalid> for crate::methods::Error {
        fn from(_: GeofenceParamsInvalid) -> Self {
            Self::invalid_params()
        }
    }

    #[derive(serde::Serialize)]
    pub struct GetGeofenceResult {
        geofence: Option<Geofence>,
    }

    impl GetGeofenceResult {
        pub fn new(geofence: Option<Geofence>) -> Self {
            Self { geofence }
        }
    }
}
