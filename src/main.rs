use dotenv::dotenv;
use futures::future;
use methods::*;
use serde_json::Value;
use std::convert::Infallible;
use std::{fmt::Debug, str::FromStr, sync::Arc};
use structopt::StructOpt;
use warp::Filter;
use warp::Reply;

mod db;
mod methods;

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
    sleep_controller: SleepController,
}

impl App {
    pub fn new() -> Self {
        let webserver_db_path: String = get_env_var("WEBSERVER_SQLITE_PATH");
        let user_db: Arc<db::Database<db::User>> =
            Arc::new(db::Database::new(webserver_db_path.clone()));
        let prediction_db: Arc<db::Database<db::Prediction>> =
            Arc::new(db::Database::new(webserver_db_path.clone()));
        Self {
            prediction_controller: PredictionController::new(prediction_db, user_db.clone()),
            user_controller: UserController::new(user_db),
            sleep_controller: SleepController::new(),
        }
    }

    /// Handle a single JSON RPC request
    async fn handle_single(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let jsonrpc = req.version().clone();
        let id = req.id().clone();
        let now = std::time::Instant::now();
        info!(
            "handling request with id {:?} with method: {}",
            id,
            req.method()
        );
        let handled_message = format!(
            "handled request with id {:?} and method: {}",
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
                Method::Sleep => JsonRpcResponse::from_result(
                    jsonrpc,
                    self.sleep_controller.sleep(req).await,
                    id,
                ),
            },
        };

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

/// Parse an environment variable as some type
pub fn get_env_var<T: FromStr>(var: &str) -> T
where
    <T as FromStr>::Err: Debug,
{
    std::env::var(var)
        .unwrap_or_else(|_| panic!(r#"could not find env var "{}""#, var))
        .parse()
        .unwrap_or_else(|_| panic!(r#"could not parse env var "{}""#, var))
}
