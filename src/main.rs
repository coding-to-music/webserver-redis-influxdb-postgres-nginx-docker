use dotenv::dotenv;
use futures::future;
use methods::*;
use serde_json::Value;
use std::convert::Infallible;
use std::{fmt::Debug, str::FromStr, sync::Arc};
use warp::Filter;
use warp::Reply;

mod methods;

#[macro_use]
extern crate log;

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

    warp::serve(handler).run(([0, 0, 0, 0], 3000)).await;
}

pub struct App {
    bookmark_controller: BookmarkController,
    prediction_controller: PredictionController,
}

impl App {
    pub fn new() -> Self {
        Self {
            bookmark_controller: BookmarkController::new(),
            prediction_controller: PredictionController::new(),
        }
    }

    async fn handle_single(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let jsonrpc = req.version().clone();
        let id = req.id().clone();
        let now = std::time::Instant::now();
        info!(
            "handling request with id {:?} with method: {}",
            id,
            req.method()
        );
        let params_clone = req.params().clone();
        let response = match Method::from_str(req.method()) {
            Err(_) => JsonRpcResponse::error(jsonrpc, Error::method_not_found(), id),
            Ok(method) => match method {
                Method::SearchBookmark => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.bookmark_controller.search(params_clone).await,
                    id,
                ),
                Method::AddBookmark => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.bookmark_controller.add(params_clone).await,
                    id,
                ),
                Method::DeleteBookmark => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.bookmark_controller.delete(params_clone).await,
                    id,
                ),
                Method::AddPrediction => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.prediction_controller.add(params_clone).await,
                    id,
                ),
            },
        };

        info!(
            "handled request with id {:?} with method: {} in {:?}",
            req.id(),
            req.method(),
            now.elapsed()
        );

        response
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

impl Default for App {
    fn default() -> Self {
        Self::new()
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
        .unwrap_or_else(|_| panic!(r#"could not find env var "{}""#, var))
        .parse()
        .unwrap_or_else(|_| panic!(r#"could not parse env var "{}""#, var))
}
