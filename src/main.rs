#![allow(dead_code)]

use controller::*;
use futures::future;
use influx::{InfluxClient, Measurement};
use serde_json::Value;
use std::{any::Any, convert::Infallible, fmt::Debug, str::FromStr, sync::Arc};
use structopt::StructOpt;
use token::TokenHandler;
use warp::{Filter, Reply};
use webserver_contracts::{
    Error as JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcVersion, Method,
};
use webserver_database::{Database, DatabaseError, ListItem as DbListItem};

mod controller;
pub mod token;

#[macro_use]
extern crate log;

#[derive(StructOpt, Debug, Clone)]
pub struct Opts {
    #[structopt(long, default_value = "3000", env = "WEBSERVER_LISTEN_PORT")]
    port: u16,
    #[structopt(long, env = "WEBSERVER_SQLITE_PATH")]
    database_path: String,
    #[structopt(long, env = "WEBSERVER_INFLUX_URL")]
    influx_url: String,
    #[structopt(long, env = "WEBSERVER_INFLUX_KEY")]
    influx_key: String,
    #[structopt(long, env = "WEBSERVER_INFLUX_ORG")]
    influx_org: String,
    #[structopt(long, env = "WEBSERVER_CERT_PATH")]
    cert_path: String,
    #[structopt(long, env = "WEBSERVER_CERT_KEY_PATH")]
    key_path: String,
    #[structopt(long, env = "WEBSERVER_REDIS_ADDR")]
    redis_addr: String,
    #[structopt(long, env = "WEBSERVER_JWT_SECRET")]
    jwt_secret: String,
}

#[tokio::main]
async fn main() {
    let env = std::env::var("WEBSERVER_ENV").unwrap_or_else(|_| "prod".to_string());

    match env.as_str() {
        "prod" => {
            dotenv::from_filename("prod.env").ok();
        }
        "test" => {
            dotenv::from_filename("test.env").ok();
        }
        invalid => panic!("invalid environment specified: '{}'", invalid),
    }

    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap())
        .init();

    let opts = Opts::from_args();

    log_opts_at_startup(&opts);

    let app = Arc::new(App::new(opts.clone()));

    let log = warp::log("api");

    let handler = warp::post()
        .and(warp::path("api"))
        .and(warp::body::json())
        .and_then(move |body| handle_request(app.clone(), body))
        .with(log);

    warp::serve(handler)
        .tls()
        .cert_path(&opts.cert_path)
        .key_path(&opts.key_path)
        .run(([0, 0, 0, 0], opts.port))
        .await;
}

fn log_opts_at_startup(opts: &Opts) {
    info!("starting webserver with opts: ");
    info!("WEBSERVER_LISTEN_PORT = {}", opts.port);
    info!("WEBSERVER_SQLITE_PATH = {}", opts.database_path);
    info!("WEBSERVER_INFLUX_URL  = {}", opts.influx_url);
    info!("WEBSERVER_INFLUX_ORG  = {}", opts.influx_org);
    info!("WEBSERVER_REDIS_ADDR  = {}", opts.redis_addr);
}

pub struct App {
    opts: Opts,
    list_controller: ListItemController,
    auth_controller: AuthController,
    influx_client: Arc<InfluxClient>,
}

impl App {
    pub fn new(opts: Opts) -> Self {
        let list_item_db: Arc<Database<DbListItem>> =
            Arc::new(Database::new(opts.database_path.clone()));

        let influx_client = Arc::new(
            InfluxClient::builder(
                opts.influx_url.to_string(),
                opts.influx_key.to_string(),
                opts.influx_org.to_string(),
            )
            .build()
            .unwrap(),
        );

        let token_handler = TokenHandler::new(opts.jwt_secret.clone());

        let list_controller = ListItemController::new(list_item_db, token_handler.clone());
        let auth_controller = AuthController::new(opts.redis_addr.to_owned(), token_handler);

        Self {
            opts,
            list_controller,
            auth_controller,
            influx_client,
        }
    }

    /// Handle a single JSON RPC request
    async fn handle_single(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let jsonrpc = request.jsonrpc;
        let id = request.id.clone();
        let method = request.method.to_owned();
        let timer = std::time::Instant::now();
        info!(
            "handling request with id {:?} with method: '{}'",
            id, request.method
        );

        let result = match Method::from_str(&method) {
            Err(_) => Err(AppError::from(JsonRpcError::method_not_found())),
            Ok(method) => {
                trace!("request: {:?}", request);
                let jsonrpc = jsonrpc.clone();
                let id = id.clone();
                match method {
                    Method::AddListItem => self
                        .list_controller
                        .add_list_item(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::GetListItems => self
                        .list_controller
                        .get_list_items(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::DeleteListItem => self
                        .list_controller
                        .delete_list_item(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::GetListTypes => self
                        .list_controller
                        .get_list_types(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::RenameListType => self
                        .list_controller
                        .rename_list_type(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::UpdateListItem => {
                        unimplemented!()
                    }
                    Method::GetToken => self
                        .auth_controller
                        .get_token(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::ValidateToken => self
                        .auth_controller
                        .validate_token(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
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
                JsonRpcResponse::error(jsonrpc, err.rpc_error, id.clone())
            }
        };

        self.log_measurement(
            Measurement::builder("handle_request")
                .with_tag("method", method)
                .with_field("duration_micros", elapsed.as_micros())
                .with_field("request_id", id.unwrap_or_default())
                .build()
                .unwrap(),
        )
        .await;

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

    async fn log_measurement(&self, measurement: Measurement) {
        let response = self
            .influx_client
            .send_batch("server", &[measurement])
            .await;
        if !response.status().is_success() {
            error!(
                "failed to send measurement to InfluxDB with status '{}'",
                response.status()
            );
        }
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

impl From<DatabaseError> for AppError {
    fn from(db_error: DatabaseError) -> Self {
        AppError::from(webserver_contracts::Error::database_error()).with_context(&db_error)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(redis_error: redis::RedisError) -> Self {
        AppError::from(webserver_contracts::Error::internal_error()).with_context(&redis_error)
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
