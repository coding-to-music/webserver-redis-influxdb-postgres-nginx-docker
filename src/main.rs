#![allow(dead_code)]

use controller::*;
use futures::future;
<<<<<<< HEAD
use hyper::{
    body::Buf,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use influx::{InfluxClient, Measurement};
use serde_json::{json, Value};
use std::{fmt::Debug, str::FromStr, sync::Arc};
use structopt::StructOpt;
use token::TokenHandler;
=======
use serde_json::Value;
use std::{any::Any, convert::Infallible, fmt::Debug, str::FromStr, sync::Arc};
use structopt::StructOpt;
use token::TokenHandler;
use warp::{Filter, Reply};
>>>>>>> master
use webserver_contracts::{
    GetTokenRequest, JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcVersion, Method,
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
<<<<<<< HEAD
    #[structopt(long, env = "WEBSERVER_SHOULD_LOG_METRICS")]
    log_metrics: bool,
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
=======
>>>>>>> master
    #[structopt(long, env = "WEBSERVER_REDIS_ADDR")]
    redis_addr: String,
    #[structopt(long, env = "WEBSERVER_JWT_SECRET")]
    jwt_secret: String,
}

#[tokio::main]
async fn main() {
    let env = std::env::var("WEBSERVER_ENV").unwrap_or_else(|_| "test".to_string());

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

    let addr = ([0, 0, 0, 0], opts.port).into();

    let service = make_service_fn(|_| {
        let app = app.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |request| {
                let app = app.clone();
                handle_request(app, request)
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

<<<<<<< HEAD
    let _ = server.await;
=======
    warp::serve(rpc).run(([0, 0, 0, 0], opts.port)).await;
>>>>>>> master
}

fn get_token(app: Arc<App>, body: Value) -> Result<String, ()> {
    match serde_json::from_value::<GetTokenRequest>(body) {
        Ok(req) => app.token_handler.get_token(&req.key_name, &req.key_value),
        Err(_serde_error) => Err(()),
    }
}

fn log_opts_at_startup(opts: &Opts) {
    info!("starting webserver with opts: ");
    info!("WEBSERVER_LISTEN_PORT = {}", opts.port);
    info!("WEBSERVER_SQLITE_PATH = {}", opts.database_path);
    info!("WEBSERVER_REDIS_ADDR  = {}", opts.redis_addr);
}

pub struct App {
    opts: Opts,
    list_controller: ListItemController,
    server_controller: ServerController,
    token_handler: Arc<TokenHandler>,
}

impl App {
    pub fn new(opts: Opts) -> Self {
        let list_item_db: Arc<Database<DbListItem>> =
            Arc::new(Database::new(opts.database_path.clone()));

        let token_handler = Arc::new(TokenHandler::new(
            opts.redis_addr.clone(),
            opts.jwt_secret.clone(),
        ));

        let list_controller = ListItemController::new(list_item_db);
        let server_controller = ServerController::new();

        Self {
            opts,
            list_controller,
            server_controller,
            token_handler,
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
                    Method::Sleep => self
                        .server_controller
                        .sleep(request)
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

<<<<<<< HEAD
        if self.opts.log_metrics {
            self.log_measurement(
                Measurement::builder("handle_request")
                    .with_tag("method", method)
                    .with_field("duration_micros", elapsed.as_micros())
                    .with_field("request_id", id.unwrap_or_default())
                    .build()
                    .unwrap(),
            )
            .await;
        }

=======
>>>>>>> master
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
    rpc_error: JsonRpcError,
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

impl From<JsonRpcError> for AppError {
    fn from(rpc_error: JsonRpcError) -> Self {
        Self {
            rpc_error,
            context: None,
        }
    }
}

impl From<DatabaseError> for AppError {
    fn from(db_error: DatabaseError) -> Self {
        AppError::from(JsonRpcError::database_error()).with_context(&db_error)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(redis_error: redis::RedisError) -> Self {
        AppError::from(JsonRpcError::internal_error()).with_context(&redis_error)
    }
}

impl From<hyper::Error> for AppError {
    fn from(hyper_error: hyper::Error) -> Self {
        AppError::from(JsonRpcError::internal_error()).with_context(&hyper_error)
    }
}

/// Process the raw JSON body of a request
/// If the request is a JSON array, handle it as a batch request
pub async fn handle_request(
    app: Arc<App>,
    request: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    match request.uri().to_string().as_str() {
        "/api" => {
            info!("api route");
            let response = match api_route(app, request).await {
                Ok(resp) => resp,
                Err(err) => {
                    let response = JsonRpcResponse::error(JsonRpcVersion::Two, err.rpc_error, None);
                    let str = serde_json::to_string(&response).unwrap();
                    let body = Body::from(str);
                    Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .body(body)
                        .unwrap()
                }
            };

            Ok(response)
        }
        "/api/token" => {
            info!("token route");
            Ok(token_route(app, request).await)
        }
        e => {
            error!("invalid route: '{}'", e);
            Ok(generic_json_response(
                json!({
                    "message": "not found"
                })
                .to_string(),
                404,
            ))
        }
    }
}

async fn token_route(app: Arc<App>, request: Request<Body>) -> Response<Body> {
    match hyper::body::aggregate(request).await {
        Ok(buf) => match serde_json::from_reader(buf.reader()) {
            Ok(req) => {
                let req: GetTokenRequest = req;
                let token = app.token_handler.get_token(&req.key_name, &req.key_value);
                match token {
                    Ok(tok) => {
                        let obj = json!({ "access_token": tok }).to_string();
                        generic_json_response(obj, 200)
                    }
                    Err(_) => {
                        let err = json!({
                            "message": "not authorized"
                        })
                        .to_string();

                        generic_json_response(err, 401)
                    }
                }
            }
            Err(_e) => {
                let err = json!({
                    "message": "invalid request"
                })
                .to_string();
                generic_json_response(err, 500)
            }
        },
        Err(_hyper_error) => {
            let err = json!({
                "message": "internal error"
            })
            .to_string();
            generic_json_response(err, 500)
        }
    }
}

async fn api_route(app: Arc<App>, request: Request<Body>) -> Result<Response<Body>, AppError> {
    match request.headers().get("Authorization") {
        Some(value) => {
            let s = value.to_str().unwrap();
            let s = s.strip_prefix("Bearer ").unwrap_or_default();
            match app.token_handler.validate_token(s) {
                Ok(_claims) => {}
                Err(_) => {
                    let err = json!({
                        "message": "not authorized"
                    })
                    .to_string();
                    return Ok(generic_json_response(err, 401));
                }
            }
        }
        None => {
            let err = json!({
                "message": "missing 'Authorization' header"
            })
            .to_string();
            return Ok(generic_json_response(err, 401));
        }
    }
    let buf = hyper::body::aggregate(request).await?;
    match serde_json::from_reader(buf.reader()) {
        Ok(json) => api_json(app, json).await,
        Err(e) => Err(AppError::from(JsonRpcError::invalid_request()).with_context(&e)),
    }
}

async fn api_json(app: Arc<App>, json: Value) -> Result<Response<Body>, AppError> {
    trace!("handling json in api route: '{}'", json);
    match json {
        Value::Array(requests) => {
            trace!("batch request");
            let rpc_requests = requests
                .into_iter()
                .map(|v| serde_json::from_value(v))
                .collect::<Result<Vec<JsonRpcRequest>, serde_json::Error>>()
                .map_err(|e| AppError::from(JsonRpcError::invalid_request()).with_context(&e))?;

            let response = app.handle_batch(rpc_requests).await;
            Ok(generic_json_response(
                serde_json::to_string(&response).unwrap(),
                200,
            ))
        }
        obj @ Value::Object(_) => {
            trace!("object request");
            let rpc_request: JsonRpcRequest = serde_json::from_value(obj)
                .map_err(|e| AppError::from(JsonRpcError::invalid_request()).with_context(&e))?;

            let response = app.handle_single(rpc_request).await;
            Ok(generic_json_response(
                serde_json::to_string(&response).unwrap(),
                200,
            ))
        }
        _ => {
            let response =
                JsonRpcResponse::error(JsonRpcVersion::Two, JsonRpcError::invalid_request(), None);
            Ok(generic_json_response(
                serde_json::to_string(&response).unwrap(),
                200,
            ))
        }
    }
}

fn generic_json_response(body: String, status: u16) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}
