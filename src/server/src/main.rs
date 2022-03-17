#![allow(clippy::new_without_default)]

use app::{App, AppError};
use auth::{Claims, TokenHandler};
use futures::future;
use hyper::{
    body::Buf,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use model::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{convert::TryInto, fmt::Debug, sync::Arc};
use structopt::StructOpt;
use time::OffsetDateTime;

pub mod app;
pub mod auth;
pub mod controller;
pub mod influx;

const API_URI: &'static str = "/api";
const PING_URI: &'static str = "/api/ping";
const URIS: [&'static str; 2] = [API_URI, PING_URI];

#[macro_use]
extern crate log;

#[derive(StructOpt, Debug, Clone)]
pub struct Opts {
    #[structopt(long, default_value = "3000", env = "WEBSERVER_LISTEN_PORT")]
    port: u16,
    #[structopt(long, env = "WEBSERVER_DATABASE_ADDR")]
    database_addr: String,
    #[structopt(long, env = "WEBSERVER_JWT_SECRET")]
    jwt_secret: String,
    #[structopt(long, env = "WEBSERVER_PUBLISH_REQUEST_LOG")]
    publish_request_log: bool,
    #[structopt(long, env = "WEBSERVER_INFLUX_ADDR")]
    influx_addr: Option<String>,
    #[structopt(long, env = "WEBSERVER_INFLUX_TOKEN")]
    influx_token: Option<String>,
    #[structopt(long, env = "WEBSERVER_INFLUX_ORG")]
    influx_org: Option<String>,
    #[structopt(long, env = "WEBSERVER_RESROBOT_API_KEY")]
    resrobot_api_key: String,
}

#[tokio::main]
async fn main() {
    let env = std::env::var("WEBSERVER_ENV").unwrap_or_else(|_| "test".to_string());

    let env_file_name = format!("{}.env", env);

    if let Err(e) = dotenv::from_filename(&env_file_name) {
        warn!(
            "environment file not found: {}, error: {}",
            env_file_name, e
        );
    }

    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&get_required_env_var("RUST_LOG"))
        .init();

    let opts = Opts::from_args();
    let tokens = Arc::new(TokenHandler::new(opts.jwt_secret.clone()));

    let app = Arc::new(App::new(opts.clone(), tokens.clone()).await);

    let webserver = Arc::new(Webserver::new(app, tokens));

    let addr = ([0, 0, 0, 0], opts.port).into();

    let service = make_service_fn(|_| {
        let webserver = webserver.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |request| {
                let webserver = webserver.clone();
                entry_point(webserver, request)
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    info!("starting server on {:?}", addr);
    let _ = server.await;
}

pub async fn entry_point(
    webserver: Arc<Webserver>,
    request: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    Ok(webserver.handle_request(request).await)
}

pub struct Webserver {
    app: Arc<App>,
    tokens: Arc<TokenHandler>,
}

impl Webserver {
    pub fn new(app: Arc<App>, tokens: Arc<TokenHandler>) -> Self {
        Self { app, tokens }
    }

    pub async fn handle_request(&self, request: Request<Body>) -> Response<Body> {
        let route = request.uri().to_string();
        let without_trailing_slash = route.trim_end_matches("/");
        // route without trailing slash for easier matching
        trace!(
            "matching route '{}' against {:?}",
            without_trailing_slash,
            URIS
        );
        match (request.method(), without_trailing_slash) {
            (_, PING_URI) => ping_pong_response(),
            (&hyper::Method::POST, API_URI) => {
                let response_body = self.api_route(request).await;
                return crate::generic_json_response(response_body, 200);
            }
            _invalid => {
                error!("invalid http method or route request: '{:?}'", request);
                return crate::generic_json_response(not_found(), 200);
            }
        }
    }

    async fn api_route(&self, request: Request<Body>) -> Vec<JsonRpcResponse> {
        let claims = self.get_auth_claims(&request);

        match Self::get_body_as_json(request).await {
            Ok(JsonValue::Array(values)) => {
                let results: Vec<_> = values
                    .into_iter()
                    .map(|v| self.parse_and_handle_single(v, &claims))
                    .collect();

                let results: Vec<_> = future::join_all(results)
                    .await
                    .into_iter()
                    .map(|res| match res {
                        Ok(response) => response,
                        Err(error) => {
                            error!("error handling request: '{:?}'", error.context);
                            Some(JsonRpcResponse::error(error.rpc_error, None))
                        }
                    })
                    .collect();
                results.into_iter().flatten().collect()
            }
            Ok(_) => {
                error!("request contains non-array JSON");
                vec![JsonRpcResponse::error(
                    JsonRpcError::invalid_request().with_message("non-array json is not supported"),
                    None,
                )]
            }
            Err(error) => {
                error!("error parsing request as json: '{:?}'", error.context);
                vec![JsonRpcResponse::error(error.rpc_error, None)]
            }
        }
    }

    fn get_auth_claims(&self, request: &Request<Body>) -> Option<Claims> {
        let header = request.headers().get("Authorization")?;
        let token = header.to_str().ok()?.trim_start_matches("Bearer ");
        self.tokens.parse_token(token).ok()
    }

    async fn parse_and_handle_single(
        &self,
        request: JsonValue,
        claims: &Option<Claims>,
    ) -> Result<Option<JsonRpcResponse>, AppError> {
        match serde_json::from_value::<JsonRpcRequest>(request) {
            Ok(request) => {
                if request.is_notification() {
                    let claims_clone = claims.clone();
                    let app = self.app.clone();
                    tokio::spawn(async move { app.handle_single(request, &claims_clone).await });
                    Ok(None)
                } else {
                    Ok(Some(self.app.handle_single(request, claims).await))
                }
            }
            Err(serde_error) => {
                Err(AppError::from(JsonRpcError::invalid_request()).with_context(&serde_error))
            }
        }
    }

    /// Attempts to parse the body of a request as json
    async fn get_body_as_json(request: Request<Body>) -> Result<JsonValue, AppError> {
        let buf = hyper::body::aggregate(request)
            .await
            .map_err(|hyper_error| AppError::invalid_request().with_context(&hyper_error))?;
        let json: JsonValue = serde_json::from_reader(buf.reader())
            .map_err(|serde_error| AppError::invalid_request().with_context(&serde_error))?;

        Ok(json)
    }
}

fn generic_json_response<T>(body: T, status: u16) -> Response<Body>
where
    T: Serialize,
{
    let b = serde_json::to_vec(&body).unwrap();

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(b))
        .unwrap()
}

fn ping_pong_response() -> Response<Body> {
    Response::builder()
        .status(200)
        .body(Body::from("pong"))
        .unwrap()
}

fn not_found() -> Vec<JsonRpcResponse> {
    let error = JsonRpcError::invalid_request().with_message("invalid route");
    let response = JsonRpcResponse::error(error, None);

    vec![response]
}

pub fn get_required_env_var(var_name: &str) -> String {
    std::env::var(var_name)
        .unwrap_or_else(|_| panic!("missing environment variable: '{}'", var_name))
}

pub fn current_timestamp_s() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp() / 1000
}

pub fn current_timestamp_ms() -> i64 {
    (OffsetDateTime::now_utc().unix_timestamp_nanos() / (1000 * 1000))
        .try_into()
        .unwrap()
}
