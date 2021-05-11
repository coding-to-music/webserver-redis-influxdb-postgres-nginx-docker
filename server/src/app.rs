use crate::{
    controller::{ListItemController, ServerController, ShapeController},
    notification::{self, NotificationHandler},
    redis::RedisPool,
    token::TokenHandler,
    Opts,
};
use chrono::Utc;
use contracts::*;
use database::{self as db, Database};
use db::DatabaseError;
use futures::future;
use hmac::crypto_mac::InvalidKeyLength;
use hyper::{body::Buf, Body, Request, Response};
use mobc_redis::redis::RedisError;
use queue::QueueMessage;
use serde_json::Value;
use std::{
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
    sync::Arc,
};

pub type AppResult<T> = Result<T, AppError>;

pub struct App {
    opts: Opts,
    list_controller: ListItemController,
    shape_controller: ShapeController,
    server_controller: ServerController,
    token_handler: Arc<TokenHandler>,
    notification_handler: Arc<NotificationHandler>,
}

impl App {
    pub fn new(opts: Opts) -> Self {
        let list_item_db: Arc<Database<db::ListItem>> =
            Arc::new(Database::new(opts.database_path.clone()));

        let shape_db: Arc<Database<db::Shape>> =
            Arc::new(Database::new(opts.database_path.clone()));

        let notification_redis_pool =
            Arc::new(RedisPool::new(opts.notification_redis_addr.clone()));
        let shape_redis_pool = Arc::new(RedisPool::new(opts.shape_redis_addr.clone()));
        let token_redis_pool = Arc::new(RedisPool::new(opts.token_redis_addr.clone()));

        let token_handler = Arc::new(TokenHandler::new(
            token_redis_pool,
            opts.jwt_secret.clone(),
        ));

        let notification_handler = Arc::new(NotificationHandler::new(
            notification::Config::new(
                crate::get_required_env_var("WEBSERVER_REDIS_NOTIFICATION_CHANNEL_PREFIX"),
                crate::get_required_env_var("WEBSERVER_REDIS_QUEUE_CHANNEL"),
            ),
            notification_redis_pool,
        ));

        let list_controller = ListItemController::new(list_item_db);
        let shape_controller = ShapeController::new(shape_redis_pool, shape_db);
        let server_controller = ServerController::new();

        Self {
            opts,
            list_controller,
            shape_controller,
            server_controller,
            token_handler,
            notification_handler,
        }
    }

    async fn handle_notification(&self, request: JsonRpcRequest) {
        let result = self
            .notification_handler
            .publish_notification(request)
            .await;

        if let Err(error) = result {
            error!(
                "error publishing notification to redis: '{:?}'",
                error.context
            );
        }
    }

    /// Handle a single JSON RPC request
    async fn handle_single(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let timer = std::time::Instant::now();
        let request_ts_s = Utc::now().timestamp();
        let id = request.id.clone();

        if id.is_none() {
            let _ = self.handle_notification(request.clone()).await;
            self.publish_request_log(
                request,
                request_ts_s,
                None,
                None,
                timer.elapsed().as_millis() as i64,
            )
            .await;
            return None;
        }

        let request_log_clone = request.clone();

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

        let (response, context) = match result {
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

        self.publish_request_log(
            request_log_clone,
            request_ts_s,
            Some(response.clone()),
            context,
            elapsed.as_millis() as i64,
        )
        .await;

        Some(response)
    }

    pub async fn handle_http_request(&self, request: Request<Body>) -> Response<Body> {
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

                self.token_handler.validate_token(token).map_err(|_| {
                    AppError::from(JsonRpcError::not_permitted().with_message("invalid token"))
                })?;

                Ok(())
            }
            None => Err(AppError::from(
                JsonRpcError::invalid_request().with_message("missing 'Authorization' header"),
            )),
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

        match get_body_as_json(request).await {
            Ok(Value::Array(values)) => {
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
        let json = get_body_as_json(request)
            .await
            .map_err(|e| GetTokenResponse::error(e.rpc_error.message))?;

        let request: GetTokenRequest = serde_json::from_value(json)
            .map_err(|serde_error| GetTokenResponse::error(serde_error.to_string()))?;

        match self
            .token_handler
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

    async fn parse_and_handle_single(
        &self,
        request: Value,
    ) -> Result<Option<JsonRpcResponse>, AppError> {
        match serde_json::from_value(request) {
            Ok(request) => {
                let request: JsonRpcRequest = request;
                Ok(self.handle_single(request).await)
            }
            Err(serde_error) => {
                Err(AppError::from(JsonRpcError::invalid_request()).with_context(&serde_error))
            }
        }
    }

    async fn publish_request_log(
        &self,
        request: JsonRpcRequest,
        request_ts_s: i64,
        response: Option<JsonRpcResponse>,
        error_context: Option<String>,
        duration_ms: i64,
    ) {
        if !self.opts.publish_request_log {
            return;
        }
        let request_log =
            QueueMessage::request_log(request, request_ts_s, response, error_context, duration_ms);
        match self
            .notification_handler
            .publish_queue_message(request_log)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("failed to publish queue message: '{}'", e)
            }
        }
    }
}

fn not_found() -> Vec<JsonRpcResponse> {
    let error = JsonRpcError::invalid_request().with_message("invalid route");
    let response = JsonRpcResponse::error(error, None);

    vec![response]
}

/// Attempts to parse the body of a request as json
async fn get_body_as_json(request: Request<Body>) -> Result<Value, AppError> {
    let buf = hyper::body::aggregate(request)
        .await
        .map_err(|hyper_error| AppError::invalid_request().with_context(&hyper_error))?;
    let json: Value = serde_json::from_reader(buf.reader())
        .map_err(|serde_error| AppError::invalid_request().with_context(&serde_error))?;

    Ok(json)
}

#[derive(Debug)]
pub struct AppError {
    rpc_error: JsonRpcError,
    context: Option<String>,
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
