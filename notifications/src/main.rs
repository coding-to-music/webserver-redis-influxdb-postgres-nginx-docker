use client::WebserverClient;
use contracts::{queue::QueueMessage, JsonRpcRequest, ResponseKind};
use database::{
    Database, Request as DbRequest, RequestLog as DbRequestLog, Response as DbResponse,
};
use futures::task::SpawnExt;
use futures_util::StreamExt;
use redis::Msg;
use serde::de::DeserializeOwned;
use std::{convert::TryFrom, sync::Arc};
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
    #[structopt(long, env = "WESBERVER_REQUEST_LOG_DB_PATH")]
    request_log_db: String,
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
    request_log_db: Database<DbRequestLog>,
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
        let request_log_db = Database::new(opts.request_log_db);

        Self {
            webserver_client,
            redis_client,
            request_log_db,
        }
    }
}

async fn run(runner: Arc<Runner>) {
    let conn = runner.redis_client.get_async_connection().await.unwrap();

    let mut ps = conn.into_pubsub();

    ps.psubscribe("ijagberg.*").await.unwrap();

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
    let channel = msg.get_channel_name();
    if channel.starts_with("ijagberg.notification") {
        match handle_notification(runner, msg).await {
            Ok(_) => {
                info!("handled notification")
            }
            Err(e) => {
                error!("failed to handle notification: {}", e);
            }
        }
    } else if channel.starts_with("ijagberg.queue") {
        match handle_queue(runner, msg).await {
            Ok(_) => {
                info!("handled queue message");
            }
            Err(e) => {
                error!("failed to handle queue message: {}", e)
            }
        }
    } else {
        warn!("received message on unsupported channel: {}", channel);
    }
}

async fn handle_notification(runner: Arc<Runner>, msg: Msg) -> Result<(), String> {
    let mut request: JsonRpcRequest = get_payload(&msg)?;
    if request.id.is_some() {
        return Err(format!("received non-notification: '{:?}'", request));
    } else {
        request.id = Some(Uuid::new_v4().to_string());
    }

    let id = request.id.clone().unwrap();

    info!(
        "sending request with id '{}' and method '{}'",
        id, request.method
    );

    let response = match runner.webserver_client.send_batch(&[request]).await {
        Ok(r) => r,
        Err(e) => {
            return Err(format!("could not send request: '{}'", e));
        }
    };

    match response.get(0) {
        Some(resp) => match resp.kind() {
            ResponseKind::Success(_) => {
                info!("received success response for request with id '{}'", id);
                return Ok(());
            }
            ResponseKind::Error(s) => {
                return Err(format!(
                    "received error response for request with id '{}': '{}'",
                    id, s.message
                ));
            }
        },
        None => {
            return Err(format!("received no response for request with id '{}'", id));
        }
    }
}

async fn handle_queue(runner: Arc<Runner>, msg: Msg) -> Result<(), String> {
    let queue_msg: QueueMessage = get_payload(&msg)?;

    match queue_msg {
        r
        @
        QueueMessage::RequestLog {
            request: _,
            request_ts_s: _,
            response: _,
            duration_ms: _,
        } => {
            let request_log = DbRequestLogWrapper::try_from(r)?.0;
            info!("saving request log: {:?}", request_log);
            runner
                .request_log_db
                .insert_log(&request_log)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn get_payload<T>(msg: &Msg) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let payload: Vec<u8> = match msg.get_payload() {
        Ok(p) => p,
        Err(e) => {
            return Err(format!("could not read payload: '{}'", e));
        }
    };
    let request: T = match serde_json::from_slice(&payload) {
        Ok(r) => r,
        Err(e) => {
            return Err(format!("could not parse json: '{}'", e));
        }
    };

    Ok(request)
}

struct DbRequestLogWrapper(DbRequestLog);

impl TryFrom<QueueMessage> for DbRequestLogWrapper {
    type Error = String;

    fn try_from(value: QueueMessage) -> Result<Self, Self::Error> {
        match value {
            QueueMessage::RequestLog {
                request,
                request_ts_s,
                response,
                duration_ms,
            } => {
                let created_s = chrono::Utc::now().timestamp();
                let params = serde_json::to_string(&request.params).map_err(|e| e.to_string())?;
                let request = DbRequest::new(request.id, request.method, params, request_ts_s);
                let (result, error) = response
                    .map(|r| match r.kind() {
                        ResponseKind::Success(v) => {
                            (Some(serde_json::to_string(&v).unwrap()), None)
                        }
                        ResponseKind::Error(e) => (None, Some(serde_json::to_string(&e).unwrap())),
                    })
                    .unwrap_or((None, None));

                let response = DbResponse::new(result, error);

                let db_request_log = DbRequestLog::new(
                    Uuid::new_v4().to_string(),
                    request,
                    response,
                    duration_ms,
                    created_s,
                );
                Ok(DbRequestLogWrapper(db_request_log))
            }
        }
    }
}
