use crate::{
    controller::{ListItemController, ServerController, ShapeController},
    redis::RedisPool,
    Opts,
};
use contracts::*;
use database::{self as db, Database};
use db::DatabaseError;
use hmac::crypto_mac::InvalidKeyLength;
use mobc_redis::redis::RedisError;
use std::{
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
    sync::Arc,
};

pub type AppResult<T> = Result<T, AppError>;

pub struct App {
    list_controller: ListItemController,
    shape_controller: ShapeController,
    server_controller: ServerController,
}

impl App {
    pub fn new(opts: Opts) -> Self {
        let list_item_db: Arc<Database<db::ListItem>> =
            Arc::new(Database::new(opts.database_path.clone()));

        let shape_db: Arc<Database<db::Shape>> =
            Arc::new(Database::new(opts.database_path.clone()));

        let shape_redis_pool = Arc::new(RedisPool::new(opts.shape_redis_addr.clone()));

        let list_controller = ListItemController::new(list_item_db);
        let shape_controller = ShapeController::new(shape_redis_pool, shape_db);
        let server_controller = ServerController::new();

        Self {
            list_controller,
            shape_controller,
            server_controller,
        }
    }

    /// Handle a single JSON RPC request
    pub async fn handle_single(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let timer = std::time::Instant::now();
        let id = request.id.clone();

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
                    Method::SearchShapesByTags => self
                        .shape_controller
                        .search_shapes_by_tags(request)
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

        let (response, _context) = match result {
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

        response
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

impl From<mobc_redis::mobc::Error<RedisError>> for AppError {
    fn from(e: mobc_redis::mobc::Error<RedisError>) -> Self {
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
