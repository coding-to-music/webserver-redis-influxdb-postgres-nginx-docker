use crate::controllers::*;
use actix_web::{body::Body, post, HttpResponse};
use bytes::Bytes;
use futures::future;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{str::FromStr, time::Instant};

/// The central structure of the webserver
///
/// Contains the logic for matching a JSON RPC method with its corresponding controller
pub struct App {
    geofences_controller: GeofencesController,
    sleep_controller: SleepController,
    gis_controller: GisController,
    positions_controller: PositionsController,
}

impl App {
    /// Construct a new App
    ///
    /// This is not intended to be used by external scopes
    fn new() -> Self {
        info!("Creating new App...");
        Self {
            geofences_controller: GeofencesController::new(),
            sleep_controller: SleepController::new(),
            gis_controller: GisController::new(),
            positions_controller: PositionsController::new(),
        }
    }

    /// Handle a single JSON RPC request
    async fn handle(&self, request: Request) -> Response {
        let id = request.id.clone();
        trace!(r#"handling request "{:?}"..."#, &id);
        let start = std::time::Instant::now();

        // match on the correct JSON RPC method
        let result = match Method::from_str(request.method.as_ref()) {
            Ok(Method::GetNearbyGeofences) => self
                .geofences_controller
                .get_nearby_geofences(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            Ok(Method::GetGeofence) => self
                .geofences_controller
                .get_geofence(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            Ok(Method::Sleep) => self
                .sleep_controller
                .sleep(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            Ok(Method::Haversine) => self
                .gis_controller
                .haversine(request.params)
                .map(|ok| serde_json::to_value(ok).unwrap()),
            Ok(Method::DistanceDriven) => self
                .positions_controller
                .get_driven_distance(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            Ok(Method::PositionHistory) => self
                .positions_controller
                .get_position_history(request.params)
                .await
                .map(|ok| serde_json::to_value(ok).unwrap()),
            Err(_) => Err(Error::method_not_found()),
        };
        trace!(
            r#"handled request "{:?}" with method "{}" in {:?}"#,
            &id,
            request.method,
            start.elapsed()
        );
        match result {
            Ok(success) => Response::success(success, id),
            Err(error) => Response::error(error, id),
        }
    }
}

/// Enum containing each supported JSON RPC method
pub enum Method {
    /// Get geofences nearby a given point
    GetNearbyGeofences,
    /// Get a single geofence by its id
    GetGeofence,
    /// Sleep for a given amount of time
    Sleep,
    /// Calculate the Haversine distance (distance on a sphere) between two coordinates
    Haversine,
    /// Calculate the distance that a vehicle has driven between two given timestamps
    DistanceDriven,
    /// Get every position a vehicle has recorded between two given timestamps
    PositionHistory,
}

impl FromStr for Method {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "get_nearby_geofences" => Self::GetNearbyGeofences,
            "get_geofence" => Self::GetGeofence,
            "sleep" => Self::Sleep,
            "haversine" => Self::Haversine,
            "distance_driven" => Self::DistanceDriven,
            "position_history" => Self::PositionHistory,
            invalid => {
                error!("invalid method: {}", invalid);
                Err(format!("invalid method: {}", invalid))?
            }
        })
    }
}

/// JSON RPC version
///
/// Currently only version 2.0 is supported by the webserver
#[derive(Serialize, Deserialize)]
pub enum Version {
    #[serde(alias = "2.0", rename = "2.0")]
    Two,
}

/// JSON RPC request
#[derive(Serialize, Deserialize)]
pub struct Request {
    jsonrpc: Version,
    method: String,
    pub(crate) params: Value,
    pub(crate) id: Option<String>,
}

/// JSON RPC response
///
/// Must contain *either* `result` or `error`
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
    /// Construct a success response (`result: Some(_)`, `error: None`)
    pub fn success<T: Into<Value>>(result: T, id: Option<String>) -> Self {
        Self {
            jsonrpc: Version::Two,
            result: Some(result.into()),
            error: None,
            id,
        }
    }

    /// Construct a success response (`result: None`, `error: Some(_)`)
    pub fn error(error: Error, id: Option<String>) -> Self {
        Self {
            jsonrpc: Version::Two,
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// Error object, used in JSON RPC responses that have failed
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

    /// Attach/overwrite a message to this `Error`
    pub fn with_message<T: Into<String>>(mut self, message: T) -> Self {
        self.message = message.into();
        self
    }

    /// Attach/overwrite data to this `Error`
    pub fn with_data<T: Into<Value>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Construct a "Method not found" `Error`
    pub fn method_not_found() -> Self {
        Self::new(ErrorCode::MethodNotFound, "Method not found".into())
    }

    /// Construct a "Parse error" `Error`
    pub fn parse_error() -> Self {
        Self::new(ErrorCode::ParseError, "Parse error".into())
    }

    /// Construct a "Invalid request" `Error`
    pub fn invalid_request() -> Self {
        Self::new(ErrorCode::InvalidRequest, "Invalid request".into())
    }

    /// Construct a "Internal error" `Error`
    pub fn internal_error() -> Self {
        Self::new(ErrorCode::InternalError, "Internal error".into())
    }

    /// Construct a "Invalid params" `Error`
    pub fn invalid_params() -> Self {
        Self::new(ErrorCode::InvalidParams, "Invalid params".into())
    }
}

/// Different kinds of errors that can occur
pub enum ErrorCode {
    /// Parse error occurs when invalid or unsupported JSON is sent
    ParseError,
    /// Invalid request occurs when the request was valid JSON, but was not a valid JSONRPC request (missing some required property)
    InvalidRequest,
    /// Method not found occurs when an invalid method is provided
    MethodNotFound,
    /// Invalid params occurs when the `params` object does not match what is expected
    InvalidParams,
    /// Internal error occurs when the server encounters an unexpected error
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
    /// Use the same App for all requests
    static ref APP: App = App::new();
}

/// Entry point for all HTTP POST requests
///
/// Handles both single requests and batches
#[post("/api")]
pub async fn handle_request(body: Bytes) -> HttpResponse {
    let json = serde_json::from_slice::<Value>(body.as_ref());
    match json {
        // If the body is a valid JSON object, then try to handle that as a single request
        Ok(obj @ Value::Object(_)) => {
            let response = handle_single(obj).await;
            HttpResponse::Ok()
                .content_type("application/json")
                .body(response)
        }
        // If the body is a valid JSON array, then try to handle that as a batch of requests
        Ok(Value::Array(arr)) => {
            let responses = handle_batch(
                arr.into_iter()
                    .map(|v| serde_json::from_value(v).unwrap())
                    .collect(),
            )
            .await;
            HttpResponse::Ok()
                .content_type("application/json")
                .body(serde_json::to_value(responses).unwrap())
        }
        // Otherwise, the body was unsupported JSON (boolean, string, etc.)
        // or not JSON at all
        _ => HttpResponse::Ok()
            .content_type("application/json")
            .body(Response::error(Error::parse_error(), None)),
    }
}

/// Attempt to deserialize a single JSON value as a JSONRPC request
async fn handle_single(value: Value) -> Response {
    // retrieve the id (if any) to be used in case of error
    let id = value
        .get("id")
        .and_then(|id| id.as_str())
        .map(|id| id.to_string());

    let response = match serde_json::from_value::<Request>(value) {
        // if the JSON value was a valid JSONRPC request, then handle that request in the App
        Ok(request) => APP.handle(request).await,
        // otherwise it was not a valid request
        Err(_) => Response::error(Error::invalid_request(), id),
    };

    response
}

/// Handle a batch of JSON values by creating a `Future` for each and awaiting them in parallell
async fn handle_batch(values: Vec<Value>) -> Vec<Response> {
    let ids: Vec<Option<&str>> = values
        .iter()
        .map(|v| v.get("id").and_then(|id| id.as_str()))
        .collect();
    let batch_id = format!("{:?}", ids);
    trace!("handling batch of requests {:?}", batch_id);
    let start = Instant::now();

    // create a `Future` for each JSON value and await them in parallell
    let responses =
        future::join_all(values.into_iter().map(|v| async { handle_single(v).await })).await;

    trace!(
        "handled batch of requests {:?} in {:?}",
        batch_id,
        start.elapsed()
    );
    responses
}
