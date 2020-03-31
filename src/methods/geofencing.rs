use geofencing_client::GeofencingClient;
use get_geofence::*;
use std::convert::TryInto;

pub struct GeofencingController {
    client: GeofencingClient,
}

impl GeofencingController {
    pub fn new() -> Self {
        info!("Creating new GeofencingController");
        let url = crate::get_env_var("IOT_GEOFENCING_API_URL");
        let key = crate::get_env_var("IOT_GEOFENCING_API_KEY");
        Self {
            client: GeofencingClient::new(url, key),
        }
    }

    pub async fn get_geofence<T>(&self, params: T) -> Result<GetGeofenceResult, super::Error>
    where
        T: TryInto<GetGeofenceParams, Error = GeofenceParamsInvalid>,
    {
        let params = params.try_into()?;
        let geofence = self
            .client
            .get_geofence_v2(params.id())
            .await
            .map_err(|_| super::Error::internal_error())?
            .map(|g| Geofence::from(g));

        Ok(GetGeofenceResult::new(geofence))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Geofence {
    id: String,
    name: String,
}

impl Geofence {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}

impl From<geofencing_client::Geofence> for Geofence {
    fn from(source: geofencing_client::Geofence) -> Geofence {
        Geofence::new(source.id().into(), source.name().into())
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
