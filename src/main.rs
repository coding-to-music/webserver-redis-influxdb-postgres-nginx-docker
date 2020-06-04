#![allow(dead_code)]

use controller::*;
use db;
use dotenv;
use futures::future;
use ring::digest;
use serde_json::Value;
use std::{
    any::Any,
    convert::Infallible,
    fmt::{Debug, Display},
    num::NonZeroU32,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};
use structopt::StructOpt;
use warp::{Filter, Reply};
use webserver_contracts::{
    Error as JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcVersion, Method,
};

mod controller;

#[macro_use]
extern crate log;

#[derive(StructOpt, Debug)]
pub struct Opts {
    #[structopt(long, default_value = "3000", env = "WEBSERVER_LISTEN_PORT")]
    port: u16,
    #[structopt(long, env = "WEBSERVER_SQLITE_PATH")]
    database_path: String,
    #[structopt(long, env = "WEBSERVER_LOG_PATH")]
    log_path: String,
}

#[tokio::main]
async fn main() {
    let env = std::env::var("WEBSERVER_ENV").unwrap_or("test".to_string());

    match env.as_str() {
        "prod" => {
            dotenv::from_filename("prod.env").ok();
        }
        "test" => {
            dotenv::from_filename("test.env").ok();
        }
        invalid => panic!("invalid environment specified: '{}'", invalid),
    }

    pretty_env_logger::init();
    let opts = Opts::from_args();

    info!("Starting webserver with opts: {:?}", opts);

    let port = opts.port;

    let app = Arc::new(App::new(opts));

    let log = warp::log("api");

    let handler = warp::post()
        .and(warp::path("api"))
        .and(warp::body::json())
        .and_then(move |body| handle_request(app.clone(), body))
        .with(log);

    warp::serve(handler).run(([0, 0, 0, 0], port)).await;
}

pub struct App {
    opts: Opts,
    prediction_controller: PredictionController,
    user_controller: UserController,
    server_controller: ServerController,
}

impl App {
    pub fn new(opts: Opts) -> Self {
        let user_db: Arc<db::Database<db::User>> =
            Arc::new(db::Database::new(opts.database_path.clone()));
        let prediction_db: Arc<db::Database<db::Prediction>> =
            Arc::new(db::Database::new(opts.database_path.clone()));
        let webserver_log_path: PathBuf = PathBuf::from(opts.log_path.clone());

        Self {
            opts,
            prediction_controller: PredictionController::new(
                prediction_db.clone(),
                user_db.clone(),
            ),
            user_controller: UserController::new(user_db.clone(), prediction_db.clone()),
            server_controller: ServerController::new(user_db, webserver_log_path),
        }
    }

    /// Handle a single JSON RPC request
    async fn handle_single(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let jsonrpc = request.version().clone();
        let id = request.id().clone();
        let method = request.method().to_owned();
        let timer = std::time::Instant::now();
        info!(
            "handling request with id {:?} with method: '{}'",
            id,
            request.method()
        );

        let result = match Method::from_str(&method) {
            Err(_) => Err(AppError::from(JsonRpcError::method_not_found())),
            Ok(method) => {
                trace!("request: {:?}", request);
                let jsonrpc = jsonrpc.clone();
                let id = id.clone();
                match method {
                    Method::AddPrediction => self
                        .prediction_controller
                        .add(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::DeletePrediction => self
                        .prediction_controller
                        .delete(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::SearchPredictions => self
                        .prediction_controller
                        .search(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::AddUser => self
                        .user_controller
                        .add(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::ChangePassword => self
                        .user_controller
                        .change_password(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::ValidateUser => self
                        .user_controller
                        .validate_user(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::DeleteUser => self
                        .user_controller
                        .delete_user(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::SetRole => self
                        .user_controller
                        .set_role(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::Sleep => self
                        .server_controller
                        .sleep(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                    Method::ClearLogs => self
                        .server_controller
                        .clear_logs(request)
                        .await
                        .map(|ok| JsonRpcResponse::success(jsonrpc, ok, id)),
                }
            }
        };

        let elapsed = timer.elapsed();
        info!(
            "handled request with id {:?} and method: '{}' in {:?}",
            id, method, elapsed
        );

        let response = match result {
            Ok(ok) => ok,
            Err(err) => {
                if err.context.is_some() {
                    error!("error with context: {:?}", err);
                }
                JsonRpcResponse::error(jsonrpc, err.rpc_error, id)
            }
        };

        crate::log_metric("handle_message_ms", elapsed.as_millis(), None);

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

#[derive(Debug)]
pub struct AppError {
    rpc_error: webserver_contracts::Error,
    context: Option<String>,
}

impl AppError {
    pub fn with_context<T>(mut self, value: &T) -> Self
    where
        T: Debug,
    {
        self.context = Some(format!("{:?}", value));
        self
    }
}

impl From<webserver_contracts::Error> for AppError {
    fn from(rpc_error: webserver_contracts::Error) -> Self {
        Self {
            rpc_error,
            context: None,
        }
    }
}

impl From<db::DatabaseError> for AppError {
    fn from(db_error: db::DatabaseError) -> Self {
        AppError::from(webserver_contracts::Error::database_error()).with_context(&db_error)
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
            JsonRpcError::invalid_request(),
            None,
        )))
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

pub fn log_metric<T>(name: &str, value: T, timestamp: Option<i64>)
where
    T: num_traits::Num + Display,
{
    info!(
        "metric:{};{};{}",
        name,
        value,
        timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp_millis())
    );
}

pub(crate) fn encrypt(
    password: &[u8],
    salt: &[u8; digest::SHA512_OUTPUT_LEN],
) -> [u8; digest::SHA512_OUTPUT_LEN] {
    let mut hash = [0u8; digest::SHA512_OUTPUT_LEN];

    ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(100_000).unwrap(),
        salt,
        password,
        &mut hash,
    );

    hash
}
