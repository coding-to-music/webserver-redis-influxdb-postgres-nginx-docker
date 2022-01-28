use crate::{
    controller::{ListItemController, ServerController, ShapeController},
    influx::InfluxClient,
    Opts,
};
use chrono::Utc;
use database::{self as db, Database};
use db::{DatabaseError, Request as DbRequest, RequestLog as DbRequestLog, Response as DbResponse};
use hmac::crypto_mac::InvalidKeyLength;
use model::*;
use redis::{mobc, redis::RedisError, RedisPool};
use std::{
    convert::TryFrom,
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
    sync::Arc,
};
use uuid::Uuid;

pub type AppResult<T> = Result<T, AppError>;

pub struct App {
    opts: Opts,
    request_log_db: Arc<Database<DbRequestLog>>,
    influx_db: Arc<InfluxClient>,
    list_controller: ListItemController,
    shape_controller: ShapeController,
    server_controller: ServerController,
}

impl App {
    pub async fn new(opts: Opts) -> Self {
        let list_item_db = Arc::new(Database::new(opts.database_addr.clone()).await.unwrap());

        let shape_db = Arc::new(Database::new(opts.database_addr.clone()).await.unwrap());

        let influx_db = Arc::new(
            InfluxClient::new(
                opts.influx_addr.clone(),
                opts.influx_token.clone(),
                opts.influx_org.clone(),
            )
            .unwrap(),
        );

        let request_log_db = Arc::new(Database::new(opts.database_addr.clone()).await.unwrap());

        let shape_redis_pool = Arc::new(RedisPool::new(opts.shape_redis_addr.clone()));

        let list_controller = ListItemController::new(list_item_db);
        let shape_controller = ShapeController::new(shape_redis_pool, shape_db);
        let server_controller = ServerController::new();

        Self {
            opts,
            request_log_db,
            list_controller,
            shape_controller,
            server_controller,
            influx_db,
        }
    }

    /// Handle a single JSON RPC request
    pub async fn handle_single(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let timer = std::time::Instant::now();
        let id = request.id.clone();
        let request_log_clone = request.clone();
        let request_ts_s = Utc::now().timestamp();

        let method = request.method.to_owned();
        info!(
            "handling request with id {:?} with method: '{}'",
            id, request.method
        );

        let result = match Method::from_str(&method) {
            Err(_) => Err(AppError::from(JsonRpcError::method_not_found())),
            Ok(method) => {
                trace!("request: {:?}", request);
                let id = id.clone();
                match method {
                    Method::AddListItem => self
                        .list_controller
                        .add_list_item(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::GetListItems => self
                        .list_controller
                        .get_list_items(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::DeleteListItem => self
                        .list_controller
                        .delete_list_item(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::GetListTypes => self
                        .list_controller
                        .get_list_types(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::RenameListType => self
                        .list_controller
                        .rename_list_type(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::Sleep => self
                        .server_controller
                        .sleep(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::AddShape => self
                        .shape_controller
                        .add_shape(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::GetShape => self
                        .shape_controller
                        .get_shape(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::GetNearbyShapes => self
                        .shape_controller
                        .get_nearby_shapes(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::DeleteShape => self
                        .shape_controller
                        .delete_shape(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::AddShapeTag => self
                        .shape_controller
                        .add_shape_tag(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::DeleteShapeTag => self
                        .shape_controller
                        .delete_shape_tag(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::RefreshGeoPointsInCache => self
                        .shape_controller
                        .refresh_geo_points_in_cache(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    Method::GenerateSasKey => self
                        .server_controller
                        .generate_sas_key(request)
                        .await
                        .map(|result| JsonRpcResponse::success(result, id)),
                    unimplemented => Ok(JsonRpcResponse::error(
                        JsonRpcError::not_implemented().with_message(format!(
                            "method '{}' is not implemented yet",
                            unimplemented.to_string()
                        )),
                        id,
                    )),
                }
            }
        };

        let elapsed = timer.elapsed();
        info!(
            "handled request with id {:?} and method: '{}' in {:?}",
            id, method, elapsed
        );

        let (response, error_context) = match result {
            Ok(ok) => (ok, None),
            Err(err) => match &err.context {
                Some(context) => {
                    error!("error with context: {:?}", err);
                    (
                        JsonRpcResponse::error(err.rpc_error, id.clone()),
                        Some(context.clone()),
                    )
                }
                None => (JsonRpcResponse::error(err.rpc_error, id.clone()), None),
            },
        };

        self.save_request_log(
            request_log_clone,
            request_ts_s,
            &response,
            error_context,
            elapsed.as_millis() as i64,
        );

        response
    }

    fn save_request_log(
        &self,
        request: JsonRpcRequest,
        request_ts_s: i64,
        response: &JsonRpcResponse,
        error_context: Option<String>,
        duration_ms: i64,
    ) {
        if !self.opts.publish_request_log {
            return;
        }

        let id = Uuid::new_v4().to_string();
        let method = request.method.clone();
        let db_request = match DbRequestWrapper::try_from((request, request_ts_s)) {
            Ok(ok) => ok.0,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        let db_response = match DbResponseWrapper::try_from(response) {
            Ok(ok) => ok.0,
            Err(err) => {
                error!("{}", err);
                return;
            }
        };
        let created_s = Utc::now().timestamp();

        let db = self.request_log_db.clone();
        tokio::spawn(async move {
            match db
                .insert_log(&DbRequestLog::new(
                    id,
                    db_request,
                    db_response,
                    error_context,
                    duration_ms,
                    created_s,
                ))
                .await
            {
                Ok(ok) => {
                    info!("successfully inserted request log with result: '{:?}'", ok);
                }
                Err(err) => {
                    error!("failed to insert request log with error: '{:?}'", err);
                }
            }
        });

        let influx = self.influx_db.clone();
        tokio::spawn(async move {
            match influx
                .send_request_log(&method, duration_ms, request_ts_s)
                .await
            {
                Ok(_) => (),
                Err(err) => {
                    error!(
                        "failed to write request log to Influx with error: '{}'",
                        err
                    );
                }
            }
        });
    }
}

#[derive(Debug)]
pub struct AppError {
    pub rpc_error: JsonRpcError,
    pub context: Option<String>,
}

impl AppError {
    pub fn with_context<T>(mut self, value: &T) -> Self
    where
        T: Debug,
    {
        self.context = Some(format!("{:?}", value));
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.rpc_error.message = message.to_owned();
        self
    }

    pub fn invalid_request() -> Self {
        Self::from(JsonRpcError::invalid_request())
    }

    pub fn invalid_params() -> Self {
        Self::from(JsonRpcError::invalid_params())
    }

    pub fn internal_error() -> Self {
        Self::from(JsonRpcError::internal_error())
    }

    pub fn database_error() -> Self {
        Self::from(JsonRpcError::database_error())
    }

    pub fn not_implemented() -> Self {
        Self::from(JsonRpcError::not_implemented())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rpc_error.message)
    }
}

impl Error for AppError {}

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
        AppError::database_error().with_context(&db_error)
    }
}

impl From<RedisError> for AppError {
    fn from(redis_error: RedisError) -> Self {
        AppError::internal_error().with_context(&redis_error)
    }
}

impl From<mobc::Error<RedisError>> for AppError {
    fn from(e: mobc::Error<RedisError>) -> Self {
        AppError::internal_error().with_context(&e)
    }
}

impl From<InvalidKeyLength> for AppError {
    fn from(e: InvalidKeyLength) -> Self {
        AppError::invalid_request().with_context(&e)
    }
}

pub trait ParamsError: Error {}

impl<T> From<T> for AppError
where
    T: ParamsError,
{
    fn from(err: T) -> Self {
        AppError::invalid_params()
            .with_message(&err.to_string())
            .with_context(&err)
    }
}

struct DbRequestWrapper(DbRequest);

impl TryFrom<(JsonRpcRequest, i64)> for DbRequestWrapper {
    type Error = String;

    fn try_from((request, ts_s): (JsonRpcRequest, i64)) -> Result<Self, Self::Error> {
        let id = request.id;
        let method = request.method;
        let params = serde_json::to_string(&request.params)
            .map_err(|_| "failed to serialize params".to_string())?;
        Ok(DbRequestWrapper(DbRequest::new(id, method, params, ts_s)))
    }
}

struct DbResponseWrapper(DbResponse);

impl TryFrom<&JsonRpcResponse> for DbResponseWrapper {
    type Error = String;

    fn try_from(value: &JsonRpcResponse) -> Result<Self, Self::Error> {
        let (result, error) = match value.kind() {
            ResponseKind::Success(s) => (
                Some(serde_json::to_string(s).map_err(|e| e.to_string())),
                None,
            ),
            ResponseKind::Error(e) => (
                None,
                Some(serde_json::to_string(e).map_err(|e| e.to_string())),
            ),
        };
        let result = result.transpose()?;
        let error = error.transpose()?;

        Ok(DbResponseWrapper(DbResponse::new(result, error)))
    }
}
