#![allow(clippy::new_without_default)]

use app::{App, AppError};
use contracts::{GetTokenRequest, GetTokenResponse, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use futures::future;
use hyper::{
    body::Buf,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use redis::RedisPool;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{fmt::Debug, sync::Arc};
use structopt::StructOpt;
use token::TokenHandler;

pub mod app;
pub mod controller;
pub mod redis;
pub mod token;

#[macro_use]
extern crate log;

#[derive(StructOpt, Debug, Clone)]
pub struct Opts {
    #[structopt(long, default_value = "3000", env = "WEBSERVER_LISTEN_PORT")]
    port: u16,
    #[structopt(long, env = "WEBSERVER_SQLITE_PATH")]
    database_path: String,
    #[structopt(long, env = "WEBSERVER_TOKEN_REDIS_ADDR")]
    token_redis_addr: String,
    #[structopt(long, env = "WEBSERVER_SHAPE_REDIS_ADDR")]
    shape_redis_addr: String,
    #[structopt(long, env = "WEBSERVER_JWT_SECRET")]
    jwt_secret: String,
    #[structopt(long, env = "WEBSERVER_PUBLISH_REQUEST_LOG")]
    publish_request_log: bool,
}

#[tokio::main]
async fn main() {
    let env = std::env::var("WEBSERVER_ENV").unwrap_or_else(|_| "test".to_string());

    match env.as_str() {
        "prod" => {
            dotenv::from_filename("prod.env").unwrap_or_else(|_| {
                panic!(
                    "prod.env not present in '{:?}'",
                    std::env::current_dir().unwrap()
                )
            });
        }
        "test" => {
            dotenv::from_filename("test.env").unwrap_or_else(|_| {
                panic!(
                    "test.env not present in '{:?}'",
                    std::env::current_dir().unwrap()
                )
            });
        }
        invalid => panic!("invalid environment specified: '{}'", invalid),
    }

    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap())
        .init();

    let opts = Opts::from_args();

    let app = Arc::new(App::new(opts.clone()));

    let webserver = Arc::new(Webserver::new(app, opts.clone()));

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
    tokens: TokenHandler,
}

impl Webserver {
    pub fn new(app: Arc<App>, opts: Opts) -> Self {
        let token_redis_pool = Arc::new(RedisPool::new(opts.token_redis_addr.clone()));
        let tokens = TokenHandler::new(token_redis_pool, opts.jwt_secret);
        Self { app, tokens }
    }

    pub async fn handle_request(&self, request: Request<Body>) -> Response<Body> {
        let route = request.uri().to_string();
        match (request.method(), route.as_str()) {
            (&hyper::Method::POST, "/api") => {
                let response_body = self.api_route(request).await;
                return crate::generic_json_response(response_body, 200);
            }
            (&hyper::Method::POST, "/api/token") => {
                let response_body = self.token_route(request).await;
                match response_body {
                    Ok(resp) | Err(resp) => {
                        return crate::generic_json_response(resp, 200);
                    }
                }
            }
            _invalid => {
                error!("invalid http method or route request: '{:?}'", request);
                return crate::generic_json_response(not_found(), 200);
            }
        }
    }

    async fn api_route(&self, request: Request<Body>) -> Vec<JsonRpcResponse> {
        if let Err(auth_error) = self.authenticate(&request) {
            error!(
                "error during authentication: '{}'",
                auth_error.rpc_error.message
            );
            return vec![JsonRpcResponse::error(auth_error.rpc_error, None)];
        }

        match Self::get_body_as_json(request).await {
            Ok(JsonValue::Array(values)) => {
                let results: Vec<_> = values
                    .into_iter()
                    .map(|v| self.parse_and_handle_single(v))
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

    async fn token_route(
        &self,
        request: Request<Body>,
    ) -> Result<GetTokenResponse, GetTokenResponse> {
        let json = Self::get_body_as_json(request)
            .await
            .map_err(|e| GetTokenResponse::error(e.rpc_error.message))?;

        let request: GetTokenRequest = serde_json::from_value(json)
            .map_err(|serde_error| GetTokenResponse::error(serde_error.to_string()))?;

        match self
            .tokens
            .get_token(&request.key_name, &request.key_value)
            .await
        {
            Ok(token) => Ok(GetTokenResponse::success(token)),
            Err(error) => {
                error!("error retrieving token: '{:?}'", error);
                Err(GetTokenResponse::error(error.rpc_error.message))
            }
        }
    }

    fn authenticate(&self, request: &Request<Body>) -> Result<(), AppError> {
        match request.headers().get("Authorization") {
            Some(value) => {
                let token = value
                    .to_str()
                    .ok()
                    .map(|tok| tok.strip_prefix("Bearer "))
                    .flatten()
                    .ok_or_else(|| {
                        AppError::from(
                            JsonRpcError::invalid_request()
                                .with_message("invalid 'Authorization' header"),
                        )
                    })?;

                self.tokens.validate_token(token).map_err(|_| {
                    AppError::from(JsonRpcError::not_permitted().with_message("invalid token"))
                })?;

                Ok(())
            }
            None => Err(AppError::from(
                JsonRpcError::invalid_request().with_message("missing 'Authorization' header"),
            )),
        }
    }

    async fn parse_and_handle_single(
        &self,
        request: JsonValue,
    ) -> Result<Option<JsonRpcResponse>, AppError> {
        match serde_json::from_value::<JsonRpcRequest>(request) {
            Ok(request) => {
                if request.is_notification() {
                    let app = self.app.clone();
                    tokio::spawn(async move { app.handle_single(request).await });
                    Ok(None)
                } else {
                    Ok(Some(self.app.handle_single(request).await))
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

fn not_found() -> Vec<JsonRpcResponse> {
    let error = JsonRpcError::invalid_request().with_message("invalid route");
    let response = JsonRpcResponse::error(error, None);

    vec![response]
}

pub fn get_required_env_var(var_name: &str) -> String {
    std::env::var(var_name)
        .unwrap_or_else(|_| panic!("missing environment variable: '{}'", var_name))
}
