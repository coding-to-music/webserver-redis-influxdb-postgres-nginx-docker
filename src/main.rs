use dotenv::dotenv;
use futures::future;
use methods::*;
use serde_json::Value;
use std::convert::Infallible;
use std::{fmt::Debug, str::FromStr, sync::Arc};
use warp::Filter;
use warp::Reply;

#[macro_use]
extern crate log;

mod methods;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let app = Arc::new(App::new());

    let log = warp::log("api");

    let handler = warp::post()
        .and(warp::path("api"))
        .and(warp::body::json())
        .and_then(move |body| handle_request(app.clone(), body))
        .with(log);

    warp::serve(handler).run(([127, 0, 0, 1], 3000)).await;
}

pub struct App {
    sleep_controller: SleepController,
    math_controller: MathController,
    geofencing_controller: GeofencingController,
    bookmark_controller: BookmarkController,
}

impl App {
    pub fn new() -> Self {
        Self {
            sleep_controller: SleepController::new(),
            math_controller: MathController::new(),
            geofencing_controller: GeofencingController::new(),
            bookmark_controller: BookmarkController::new(),
        }
    }

    async fn handle_single(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let version = req.version().clone();
        let id = req.id().clone();
        info!("method: {}", req.method());
        match Method::from_str(req.method()) {
            Err(_) => JsonRpcResponse::error(version, Error::method_not_found(), id),
            Ok(method) => match method {
                Method::Sleep => JsonRpcResponse::from_result(
                    version,
                    self.sleep_controller.sleep(req.params().to_owned()).await,
                    id,
                ),
                Method::Add => JsonRpcResponse::from_result(
                    version,
                    self.math_controller.add(req.params().to_owned()),
                    id,
                ),
                Method::Subtract => JsonRpcResponse::from_result(
                    version,
                    self.math_controller.subtract(req.params().to_owned()),
                    id,
                ),
                Method::GetGeofence => JsonRpcResponse::from_result(
                    version,
                    self.geofencing_controller
                        .get_geofence(req.params().to_owned())
                        .await,
                    id,
                ),
                Method::SearchBookmark => JsonRpcResponse::from_result(
                    version,
                    self.bookmark_controller
                        .search(req.params().to_owned())
                        .await,
                    id,
                ),
            },
        }
    }

    async fn handle_batch(&self, reqs: Vec<JsonRpcRequest>) -> Vec<JsonRpcResponse> {
        future::join_all(
            reqs.into_iter()
                .map(|req| self.handle_single(req))
                .collect::<Vec<_>>(),
        )
        .await
    }
}

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

pub fn get_env_var<T: FromStr>(var: &str) -> T
where
    <T as FromStr>::Err: Debug,
{
    std::env::var(var)
        .expect(&format!(r#"could not find env var "{}""#, var))
        .parse()
        .expect(&format!(r#"could not parse env var "{}""#, var))
}
