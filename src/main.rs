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
    Error, JsonRpcRequest, JsonRpcResponse, JsonRpcVersion, Method, ResponseKind,
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
            prediction_controller: PredictionController::new(prediction_db, user_db.clone()),
            user_controller: UserController::new(user_db.clone()),
            server_controller: ServerController::new(user_db, webserver_log_path),
        }
    }

    /// Handle a single JSON RPC request
    async fn handle_single(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let jsonrpc = req.version().clone();
        let id = req.id().clone();
        let timer = std::time::Instant::now();
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
                Method::DeleteUser => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.user_controller.delete_user(req).await,
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
                Method::ClearLogs => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.server_controller.clear_logs(req).await,
                    id,
                ),
            },
        };

        if let ResponseKind::Error(err) = response.kind() {
            if let Some(data) = err.get_internal_data() {
                error!("returning an error with internal data: '{}'", data);
            }
        }

        let elapsed = timer.elapsed();
        crate::log_metric("handle_message_ms", elapsed.as_millis(), None);
        info!("{} in {:?}", handled_message, elapsed);

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
