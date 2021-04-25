use app::{App, AppError};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use serde::Serialize;
use std::{fmt::Debug, sync::Arc};
use structopt::StructOpt;

pub mod app;
pub mod controller;
pub mod notification;
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
    #[structopt(long, env = "WEBSERVER_NOTIFICATION_REDIS_ADDR")]
    notification_redis_addr: String,
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
            dotenv::from_filename("prod.env").expect(&format!(
                "prod.env not present in '{:?}'",
                std::env::current_dir().unwrap()
            ));
        }
        "test" => {
            dotenv::from_filename("test.env").expect(&format!(
                "test.env not present in '{:?}'",
                std::env::current_dir().unwrap()
            ));
        }
        invalid => panic!("invalid environment specified: '{}'", invalid),
    }

    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap())
        .init();

    let opts = Opts::from_args();

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

    info!("starting server on {:?}", addr);
    let _ = server.await;
}

/// Process the raw JSON body of a request
/// If the request is a JSON array, handle it as a batch request
pub async fn handle_request(
    app: Arc<App>,
    request: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    Ok(app.handle_http_request(request).await)
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
