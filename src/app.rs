use crate::{
    controller::{ListItemController, ServerController, ShapeController},
    notification::NotificationHandler,
    token::TokenHandler,
    Opts,
};
use db::DatabaseError;
use futures::future;
use hyper::{body::Buf, Body, Request, Response};
use serde_json::Value;
use std::{fmt::Debug, str::FromStr, sync::Arc};
use webserver_contracts::{
    GetTokenRequest, GetTokenResponse, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    JsonRpcVersion, Method,
};
use webserver_database::{self as db, Database};

pub struct App {
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

        let token_handler = Arc::new(TokenHandler::new(
            opts.notification_redis_addr.clone(),
            opts.jwt_secret.clone(),
        ));

        let notification_handler = Arc::new(NotificationHandler::new(
            opts.notification_redis_addr.clone(),
        ));

        let list_controller = ListItemController::new(list_item_db);
        let shape_controller = ShapeController::new(opts.shape_redis_addr, shape_db.clone());
        let server_controller = ServerController::new();

        Self {
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
        let jsonrpc = request.jsonrpc;
        let id = request.id.clone();

        if id.is_none() {
            let _ = self.handle_notification(request).await;
            return None;
        }

        let method = request.method.to_owned();
        let timer = std::time::Instant::now();
        info!(
            "handling request with id {:?} with method: '{}'",
            id, request.method
        );

        let result = match Method::from_str(&method) {
            Err(_) => Err(AppError::from(JsonRpcError::method_not_found())),
            Ok(method) => {
                trace!("request: {:?}", request);
                let jsonrpc = jsonrpc.clone();
                let id = id.clone();
                match method {
                    Method::AddListItem => self
                        .list_controller
                        .add_list_item(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::GetListItems => self
                        .list_controller
                        .get_list_items(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::DeleteListItem => self
                        .list_controller
                        .delete_list_item(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::GetListTypes => self
                        .list_controller
                        .get_list_types(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::RenameListType => self
                        .list_controller
                        .rename_list_type(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::Sleep => self
                        .server_controller
                        .sleep(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::AddShape => self
                        .shape_controller
                        .add_shape(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    Method::GetShape => self
                        .shape_controller
                        .get_shape(request)
                        .await
                        .map(|result| JsonRpcResponse::success(jsonrpc, result, id)),
                    unimplemented => Ok(JsonRpcResponse::error(
                        JsonRpcVersion::Two,
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

        let response = match result {
            Ok(ok) => ok,
            Err(err) => {
                if err.context.is_some() {
                    error!("error with context: {:?}", err);
                }
                JsonRpcResponse::error(jsonrpc, err.rpc_error, id.clone())
            }
        };

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
                    .ok_or(AppError::from(
                        JsonRpcError::invalid_request()
                            .with_message("invalid 'Authorization' header"),
                    ))?;

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
            return vec![JsonRpcResponse::error(
                JsonRpcVersion::Two,
                auth_error.rpc_error,
                None,
            )];
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
                            Some(JsonRpcResponse::error(
                                JsonRpcVersion::Two,
                                error.rpc_error,
                                None,
                            ))
                        }
                    })
                    .collect();
                results.into_iter().filter_map(|res| res).collect()
            }
            Ok(_) => {
                error!("request contains non-array JSON");
                vec![JsonRpcResponse::error(
                    JsonRpcVersion::Two,
                    JsonRpcError::invalid_request().with_message("non-array json is not supported"),
                    None,
                )]
            }
            Err(error) => {
                error!("error parsing request as json: '{:?}'", error.context);
                vec![JsonRpcResponse::error(
                    JsonRpcVersion::Two,
                    error.rpc_error,
                    None,
                )]
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
}

fn not_found() -> Vec<JsonRpcResponse> {
    let error = JsonRpcError::invalid_request().with_message("invalid route");
    let response = JsonRpcResponse::error(JsonRpcVersion::Two, error, None);

    vec![response]
}

/// Attempts to parse the body of a request as json
async fn get_body_as_json(request: Request<Body>) -> Result<Value, AppError> {
    let buf = hyper::body::aggregate(request)
        .await
        .map_err(|hyper_error| {
            AppError::from(JsonRpcError::invalid_request()).with_context(&hyper_error)
        })?;
    let json: Value = serde_json::from_reader(buf.reader()).map_err(|serde_error| {
        AppError::from(JsonRpcError::invalid_request()).with_context(&serde_error)
    })?;

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

    pub fn invalid_params() -> Self {
        Self::from(JsonRpcError::invalid_params())
    }

    pub fn internal_error() -> Self {
        Self::from(JsonRpcError::internal_error())
    }
}

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
        AppError::from(JsonRpcError::database_error()).with_context(&db_error)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(redis_error: redis::RedisError) -> Self {
        AppError::from(JsonRpcError::internal_error()).with_context(&redis_error)
    }
}
