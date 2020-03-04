use crate::controllers::*;
use actix_web::{body::Body, post, HttpResponse};
use bytes::Bytes;
use futures::future;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct App {
    geofences_controller: GeofencesController,
    sleep_controller: SleepController,
    gis_controller: GisController,
    positions_controller: PositionsController,
}

impl App {
    pub fn new() -> Self {
        info!("Creating new App...");
        Self {
            geofences_controller: GeofencesController::new(),
            sleep_controller: SleepController::new(),
            gis_controller: GisController::new(),
            positions_controller: PositionsController::new(),
        }
    }

    async fn handle(&self, request: Request) -> Response {
        let id = request.id.clone();
        trace!(r#"handling request "{:?}"..."#, &id);
        let start = std::time::Instant::now();
        let result = match request.method.as_ref() {
            "get_nearby_geofences" => self
                .geofences_controller
                .get_nearby_geofences(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            "get_geofence" => self
                .geofences_controller
                .get_geofence(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            "sleep" => self
                .sleep_controller
                .sleep(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            "haversine" => self
                .gis_controller
                .haversine(request.params)
                .map(|ok| serde_json::to_value(ok).unwrap()),
            "distance_driven" => self
                .positions_controller
                .get_driven_distance(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            _ => Err(Error::method_not_found()),
        };
        trace!(r#"handled request "{:?}" in {:?}"#, &id, start.elapsed());
        match result {
            Ok(success) => Response::success(success, id),
            Err(error) => Response::error(error, id),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Version {
    #[serde(alias = "2.0", rename = "2.0")]
    Two,
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    jsonrpc: Version,
    method: String,
    pub(crate) params: Value,
    pub(crate) id: Option<String>,
}

#[derive(Serialize)]
pub struct Response {
    jsonrpc: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
    id: Option<String>,
}

impl Response {
    pub fn success<T: Into<Value>>(result: T, id: Option<String>) -> Self {
        Self {
            jsonrpc: Version::Two,
            result: Some(result.into()),
            error: None,
            id,
        }
    }

    pub fn error(error: Error, id: Option<String>) -> Self {
        Self {
            jsonrpc: Version::Two,
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Error {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl Error {
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code: code.into(),
            message,
            data: None,
        }
    }

    pub fn with_message<T: Into<String>>(mut self, message: T) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_data<T: Into<Value>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
    }

    pub fn method_not_found() -> Self {
        Self::new(ErrorCode::MethodNotFound, "Method not found".into())
    }

    pub fn parse_error() -> Self {
        Self::new(ErrorCode::ParseError, "Parse error".into())
    }

    pub fn invalid_request() -> Self {
        Self::new(ErrorCode::InvalidRequest, "Invalid request".into())
    }

    pub fn internal_error() -> Self {
        Self::new(ErrorCode::InternalError, "Internal error".into())
    }

    pub fn invalid_params() -> Self {
        Self::new(ErrorCode::InvalidParams, "Invalid params".into())
    }
}

pub enum ErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
}

impl From<ErrorCode> for i32 {
    fn from(error_code: ErrorCode) -> Self {
        match error_code {
            ErrorCode::ParseError => -32700,
            ErrorCode::InvalidRequest => -32600,
            ErrorCode::MethodNotFound => -32601,
            ErrorCode::InvalidParams => -32602,
            ErrorCode::InternalError => -32603,
        }
    }
}

impl Into<Body> for Response {
    fn into(self) -> Body {
        serde_json::to_string(&self).unwrap().into()
    }
}

lazy_static! {
    static ref APP: App = App::new();
}

#[post("/api")]
pub async fn handle_request(body: Bytes) -> HttpResponse {
    HttpResponse::Ok().content_type("application/json").body(
        match serde_json::from_slice::<Request>(body.as_ref()) {
            Ok(request) => APP.handle(request).await,
            Err(_) => Response::error(Error::invalid_request(), None),
        },
    )
}

#[post("api/batch")]
pub async fn handle_request_batch(body: Bytes) -> HttpResponse {
    let values = match serde_json::from_slice::<Vec<Value>>(body.as_ref()) {
        Ok(values) => values,
        Err(_) => {
            return HttpResponse::Ok()
                .content_type("application/json")
                .body(Response::error(Error::parse_error(), None))
        }
    };

    let responses = future::join_all(values.into_iter().map(|v| async {
        let id = v
            .get("id")
            .and_then(|id| id.as_str())
            .map(|id| id.to_string());

        match serde_json::from_value::<Request>(v) {
            Ok(request) => APP.handle(request).await,
            Err(_) => Response::error(Error::invalid_request(), id),
        }
    }))
    .await;

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&responses).unwrap())
}
