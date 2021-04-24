use client::WebserverClient;
use contracts::{JsonRpcRequest, ResponseKind};
use futures::task::SpawnExt;
use futures_util::StreamExt as _;
use log::warn;
use redis::Msg;
use std::sync::Arc;
use structopt::StructOpt;
use uuid::Uuid;

#[macro_use]
extern crate log;

#[derive(Debug, Clone, StructOpt)]
struct Opts {
    #[structopt(long, env = "WEBSERVER_REDIS_ADDRESS")]
    redis_addr: String,
    #[structopt(long, env = "WEBSERVER_ADDRESS")]
    webserver_addr: String,
    #[structopt(long, env = "WEBSERVER_KEY_NAME")]
    webserver_key_name: String,
    #[structopt(long, env = "WEBSERVER_KEY_VALUE")]
    webserver_key_value: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let opts = Opts::from_args();

    let runner = Arc::new(Runner::new(opts));

    run(runner).await
}

struct Runner {
    webserver_client: WebserverClient,
    redis_client: redis::Client,
}

impl Runner {
    fn new(opts: Opts) -> Self {
        let webserver_client = WebserverClient::new(
            opts.webserver_addr.clone(),
            opts.webserver_key_name.clone(),
            opts.webserver_key_value.clone(),
        )
        .build()
        .unwrap();
        let redis_client = redis::Client::open(opts.redis_addr.clone()).unwrap();

        Self {
            webserver_client,
            redis_client,
        }
    }
}

async fn run(runner: Arc<Runner>) {
    let conn = runner.redis_client.get_async_connection().await.unwrap();

    let mut ps = conn.into_pubsub();

    ps.psubscribe("ijagberg.notification.*").await.unwrap();

    let mut stream = ps.on_message();

    let pool = futures::executor::ThreadPool::new().unwrap();
    loop {
        let msg = match stream.next().await {
            Some(msg) => msg,
            None => {
                warn!("nothing in stream");
                continue;
            }
        };

        let task = handle_message(runner.clone(), msg);
        pool.spawn(task).unwrap();
    }
}

async fn handle_message(runner: Arc<Runner>, msg: Msg) {
    let payload: Vec<u8> = match msg.get_payload() {
        Ok(p) => p,
        Err(e) => {
            error!("could not read payload: '{}'", e);
            return;
        }
    };

    let mut request: JsonRpcRequest = match serde_json::from_slice(&payload) {
        Ok(r) => r,
        Err(e) => {
            error!("could not parse json: '{}'", e);
            return;
        }
    };

    if request.id.is_some() {
        error!("received non-notification: '{:?}'", request);
        return;
    } else {
        request.id = Some(Uuid::new_v4().to_string());
    }

    let id = request.id.clone().unwrap();

    info!(
        "sending request with id '{}' and method '{}'",
        id, request.method
    );

    let response = match runner.webserver_client.send_batch(vec![request]).await {
        Ok(r) => r,
        Err(e) => {
            error!("could not send request: '{}'", e);
            return;
        }
    };

    match response.get(0) {
        Some(resp) => match resp.kind() {
            ResponseKind::Success(_) => {
                info!("received success response for request with id '{}'", id);
            }
            ResponseKind::Error(s) => {
                error!(
                    "received error response for request with id '{}': '{}'",
                    id, s.message
                );
            }
        },
        None => {
            error!("received no response for request with id '{}'", id);
        }
    }
}
