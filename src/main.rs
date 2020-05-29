mod db;
mod methods;

use dotenv::dotenv;
use futures::future;
use methods::{Method, PredictionController, ServerController, UserController};
use serde::Serialize;
use serde_json::Value;
use std::{any::Any, convert::Infallible, fmt::Debug, str::FromStr, sync::Arc};
use structopt::StructOpt;
use warp::{Filter, Reply};

#[macro_use]
extern crate log;

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    let opts = Opts::from_args();

    let app = Arc::new(App::new());

    let log = warp::log("api");

    let handler = warp::post()
        .and(warp::path("api"))
        .and(warp::body::json())
        .and_then(move |body| handle_request(app.clone(), body))
        .with(log);

    warp::serve(handler).run(([0, 0, 0, 0], opts.port)).await;
}

pub struct App {
    prediction_controller: PredictionController,
    user_controller: UserController,
    server_controller: ServerController,
}

impl App {
    pub fn new() -> Self {
        let webserver_db_path: String = from_env_var("WEBSERVER_SQLITE_PATH").unwrap();
        let user_db: Arc<db::Database<db::User>> =
            Arc::new(db::Database::new(webserver_db_path.clone()));
        let prediction_db: Arc<db::Database<db::Prediction>> =
            Arc::new(db::Database::new(webserver_db_path));
        Self {
            prediction_controller: PredictionController::new(prediction_db, user_db.clone()),
            user_controller: UserController::new(user_db.clone()),
            server_controller: ServerController::new(user_db),
        }
    }

    /// Handle a single JSON RPC request
    async fn handle_single(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let jsonrpc = req.version().clone();
        let id = req.id().clone();
        let now = std::time::Instant::now();
        info!(
            "handling request with id {:?} with method: '{}'",
            id,
            req.method()
        );
        let handled_message = format!(
            "handled request with id {:?} and method: '{}'",
            req.id(),
            req.method()
        );
        let response = match Method::from_str(req.method()) {
            Err(_) => JsonRpcResponse::error(jsonrpc, Error::method_not_found(), id),
            Ok(method) => match method {
                Method::AddPrediction => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.prediction_controller.add(req).await,
                    id,
                ),
                Method::DeletePrediction => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.prediction_controller.delete(req).await,
                    id,
                ),
                Method::SearchPredictions => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.prediction_controller.search(req).await,
                    id,
                ),
                Method::AddUser => {
                    JsonRpcResponse::from_result(jsonrpc, self.user_controller.add(req).await, id)
                }
                Method::ChangePassword => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.user_controller.change_password(req).await,
                    id,
                ),
                Method::ValidateUser => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.user_controller.validate_user(req).await,
                    id,
                ),
                Method::SetRole => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.user_controller.set_role(req).await,
                    id,
                ),
                Method::Sleep => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.server_controller.sleep(req).await,
                    id,
                ),
            },
        };

        if let ResponseKind::Error(err) = response.kind() {
            if let Some(data) = err.get_internal_data() {
                error!("returning an error with internal data: '{}'", data);
            }
        }

        info!("{} in {:?}", handled_message, now.elapsed());

        response
    }

    /// Handle multiple JSON RPC requests concurrently by awaiting them all
    async fn handle_batch(&self, reqs: Vec<JsonRpcRequest>) -> Vec<JsonRpcResponse> {
        future::join_all(
            reqs.into_iter()
                .map(|req| self.handle_single(req))
                .collect::<Vec<_>>(),
        )
        .await
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Process the raw JSON body of a request
/// If the request is a JSON array, handle it as a batch request
pub async fn handle_request(app: Arc<App>, body: Value) -> Result<impl Reply, Infallible> {
    if body.is_object() {
        Ok(warp::reply::json(
            &app.handle_single(serde_json::from_value(body).unwrap())
                .await,
        ))
    } else if let Value::Array(values) = body {
        Ok(warp::reply::json(
            &app.handle_batch(
                values
                    .into_iter()
                    .map(|value| serde_json::from_value(value).unwrap())
                    .collect(),
            )
            .await,
        ))
    } else {
        Ok(warp::reply::json(&JsonRpcResponse::error(
            JsonRpcVersion::Two,
            Error::invalid_request(),
            None,
        )))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum JsonRpcVersion {
    #[serde(alias = "1.0", rename = "1.0")]
    One,
    #[serde(alias = "2.0", rename = "2.0")]
    Two,
}

/// A JSONRPC request.
#[derive(serde::Deserialize)]
pub struct JsonRpcRequest {
    /// JSONRPC version.
    jsonrpc: JsonRpcVersion,
    /// RPC method to call.
    method: String,
    /// Parameters to pass to the method.
    params: Value,
    /// A response to this request should contain this same id (provided by the requester).
    /// If the request is a notification, then `id` is `None`.
    id: Option<String>,
}

impl JsonRpcRequest {
    pub fn version(&self) -> &JsonRpcVersion {
        &self.jsonrpc
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn id(&self) -> &Option<String> {
        &self.id
    }
}

/// A JSONRPC response object. Contains _either_ a `result` (in case of success) or `error` (in case of failure) property.
#[derive(serde::Serialize)]
pub struct JsonRpcResponse {
    /// JSONRPC version of the response.
    jsonrpc: JsonRpcVersion,
    /// Optional structured data to be returned in case of success
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    /// Optional data to be returned in case of failure
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
    /// Id corresponding to `id` property of request (if any)
    id: Option<String>,
}

impl JsonRpcResponse {
    pub fn kind(&self) -> ResponseKind {
        if let Some(err) = &self.error {
            ResponseKind::Error(err)
        } else {
            ResponseKind::Success
        }
    }

    /// Create a `JsonRpcResponse` from a `Result`.
    pub fn from_result<T>(
        jsonrpc: JsonRpcVersion,
        result: Result<T, Error>,
        id: Option<String>,
    ) -> Self
    where
        T: Serialize,
    {
        match result {
            Ok(s) => Self::success(jsonrpc, s, id),
            Err(e) => Self::error(jsonrpc, e, id),
        }
    }

    /// Create a `JsonRpcResponse` with a `result` property (indicating success).
    pub fn success<T: Serialize>(jsonrpc: JsonRpcVersion, result: T, id: Option<String>) -> Self {
        Self {
            jsonrpc,
            result: Some(serde_json::to_value(result).expect("infallible")),
            error: None,
            id,
        }
    }

    /// Create a `JsonRpcResponse` with an `error` property (indicating failure).
    pub fn error(jsonrpc: JsonRpcVersion, error: Error, id: Option<String>) -> Self {
        Self {
            jsonrpc,
            result: None,
            error: Some(error),
            id,
        }
    }

    pub fn get_error(&self) -> Option<&Error> {
        match &self.error {
            None => None,
            Some(err) => Some(&err),
        }
    }
}

pub enum ResponseKind<'a> {
    Success,
    Error(&'a Error),
}

/// Error object to be returned in a `JsonRpcResponse` if something failed.
#[derive(Serialize)]
pub struct Error {
    /// JSONRPC error code.
    code: i32,
    /// Short description of what went wrong.
    message: String,
    /// Optional field containing structured error information.
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    /// Contains debug information about what caused an error.
    /// This property is not exposed to callers of the api.
    #[serde(skip_serializing)]
    internal_data: Option<String>,
}

impl Error {
    /// Set the `message` property on `self`.
    pub fn with_message<T>(mut self, message: T) -> Self
    where
        T: Into<String>,
    {
        self.message = message.into();
        self
    }

    /// Set the `data` property on `self`.
    pub fn with_data<T: Serialize>(mut self, data: T) -> Self {
        self.data = Some(serde_json::to_value(data).expect("infallible"));
        self
    }

    /// Set the `internal_data` property on `self`.
    pub fn with_internal_data<T: Debug>(mut self, data: T) -> Self {
        self.internal_data = Some(format!("{:?}", data));
        self
    }

    /// Constructor for a "Method not found" JSONRPC error.
    pub fn method_not_found() -> Self {
        Self {
            code: ErrorCode::MethodNotFound.into(),
            message: "Method not found".into(),
            data: None,
            internal_data: None,
        }
    }

    /// Constructor for a "Invalid request" JSONRPC error.
    pub fn invalid_request() -> Self {
        Self {
            code: ErrorCode::InvalidRequest.into(),
            message: "Invalid request".into(),
            data: None,
            internal_data: None,
        }
    }

    /// Constructor for an "Invalid params" JSONRPC error.
    pub fn invalid_params() -> Self {
        Self {
            code: ErrorCode::InvalidParams.into(),
            message: "Invalid params".into(),
            data: None,
            internal_data: None,
        }
    }

    /// Constructor for an "Internal error" JSONRPC error.
    pub fn internal_error() -> Self {
        Self {
            code: ErrorCode::InternalError.into(),
            message: "Internal error".into(),
            data: None,
            internal_data: None,
        }
    }

    pub fn get_internal_data(&self) -> Option<&str> {
        match &self.internal_data {
            None => None,
            Some(data) => Some(data),
        }
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::internal_error()
            .with_data("database error")
            .with_internal_data(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::internal_error()
            .with_data("http error")
            .with_internal_data(e)
    }
}

/// Standard JSONRPC error variants as defined by the [JSONRPC specification](https://www.jsonrpc.org/specification#error_object)
pub enum ErrorCode {
    /// Invalid JSON was received.
    ParseError,
    /// The JSON received was not a valid JSONRPC request object.
    InvalidRequest,
    /// The method does not exist / is not available.
    MethodNotFound,
    /// Invalid method parameter(s).
    InvalidParams,
    /// Internal JSONRPC error
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

/// Parse an environment variable as some type
pub fn from_env_var<T: FromStr + Any>(var: &str) -> Result<T, String>
where
    <T as FromStr>::Err: Debug,
{
    std::env::var(var)
        .map_err(|_| format!("could not find env var '{}'", var))?
        .parse::<T>()
        .map_err(|_| {
            format!(
                "could not parse env var '{}' as '{}'",
                var,
                std::any::type_name::<T>()
            )
        })
}
