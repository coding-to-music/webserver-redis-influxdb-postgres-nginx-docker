#![allow(dead_code)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{
    convert::Infallible,
    fmt::{Debug, Display},
    str::FromStr,
};

pub mod prediction;
pub mod server;
pub mod user;

const SLEEP: &str = "sleep";
const ADD_PREDICTION: &str = "add_prediction";
const DELETE_PREDICTION: &str = "delete_prediction";
const SEARCH_PREDICTION: &str = "search_predictions";
const ADD_USER: &str = "add_user";
const CHANGE_PASSWORD: &str = "change_password";
const VALIDATE_USER: &str = "validate_user";
const SET_ROLE: &str = "set_role";
const CLEAR_LOGS: &str = "clear_logs";
const DELETE_USER: &str = "delete_user";
const PREPARE_TESTS: &str = "prepare_tests";

/// A JSONRPC method
pub enum Method {
    /// Sleep for a specified amount of time
    Sleep,
    /// Add a prediction to the database
    AddPrediction,
    /// Delete a prediction by its database id
    DeletePrediction,
    /// Search predictions
    SearchPredictions,
    /// Add a user
    AddUser,
    /// Change password for a user
    ChangePassword,
    /// Validate a username, password tuple
    ValidateUser,
    /// Delete a user
    DeleteUser,
    /// Set the role of a given user
    SetRole,
    /// Clear webserver logs
    ClearLogs,
    /// Prepare the webserver for integration tests
    PrepareTests,
}

impl FromStr for Method {
    type Err = (); // any failure means the method simply doesn't exist
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ADD_PREDICTION => Ok(Self::AddPrediction),
            DELETE_PREDICTION => Ok(Self::DeletePrediction),
            SEARCH_PREDICTION => Ok(Self::SearchPredictions),
            ADD_USER => Ok(Self::AddUser),
            CHANGE_PASSWORD => Ok(Self::ChangePassword),
            VALIDATE_USER => Ok(Self::ValidateUser),
            SET_ROLE => Ok(Self::SetRole),
            SLEEP => Ok(Self::Sleep),
            CLEAR_LOGS => Ok(Self::ClearLogs),
            DELETE_USER => Ok(Self::DeleteUser),
            PREPARE_TESTS => Ok(Self::PrepareTests),
            _ => Err(()),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Sleep => SLEEP,
                Method::AddPrediction => ADD_PREDICTION,
                Method::DeletePrediction => DELETE_PREDICTION,
                Method::SearchPredictions => SEARCH_PREDICTION,
                Method::AddUser => ADD_USER,
                Method::ChangePassword => CHANGE_PASSWORD,
                Method::ValidateUser => VALIDATE_USER,
                Method::SetRole => SET_ROLE,
                Method::ClearLogs => CLEAR_LOGS,
                Method::DeleteUser => DELETE_USER,
                Method::PrepareTests => PREPARE_TESTS,
            }
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum JsonRpcVersion {
    #[serde(alias = "1.0", rename = "1.0")]
    One,
    #[serde(alias = "2.0", rename = "2.0")]
    Two,
}

/// A JSONRPC request.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct JsonRpcRequest {
    /// JSONRPC version.
    jsonrpc: JsonRpcVersion,
    /// RPC method to call.
    method: String,
    /// Parameters to pass to the method.
    params: Value,
    /// A response to this request should contain this same id (provided by the requester).
    /// If the request is a notification, then `id` is `None`.
    id: Option<String>,
}

impl JsonRpcRequest {
    pub fn new<T>(jsonrpc: JsonRpcVersion, method: String, params: T, id: Option<String>) -> Self
    where
        T: Serialize,
    {
        Self {
            jsonrpc,
            method,
            params: serde_json::to_value(params).unwrap(),
            id,
        }
    }

    pub fn version(&self) -> &JsonRpcVersion {
        &self.jsonrpc
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn id(&self) -> &Option<String> {
        &self.id
    }
}

pub struct JsonRpcRequestBuilder {
    jsonrpc: Option<JsonRpcVersion>,
    method: Option<String>,
    params: Option<Value>,
    id: Option<String>,
}

pub enum JsonRpcRequestBuilderError {
    MissingMethod,
    MissingParams,
}

impl Display for JsonRpcRequestBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            JsonRpcRequestBuilderError::MissingMethod => "missing 'method' property",
            JsonRpcRequestBuilderError::MissingParams => "missing 'params' property",
        };
        write!(f, "{}", output)
    }
}

impl From<JsonRpcRequestBuilderError> for String {
    fn from(error: JsonRpcRequestBuilderError) -> Self {
        format!("{}", error)
    }
}

impl JsonRpcRequestBuilder {
    pub fn new() -> Self {
        Self {
            jsonrpc: None,
            method: None,
            params: None,
            id: None,
        }
    }

    pub fn build(self) -> Result<JsonRpcRequest, JsonRpcRequestBuilderError> {
        let jsonrpc = self.jsonrpc.unwrap_or(JsonRpcVersion::Two);
        let method = self
            .method
            .ok_or(JsonRpcRequestBuilderError::MissingMethod)?;
        let params = self
            .params
            .ok_or(JsonRpcRequestBuilderError::MissingParams)?;
        let id = self.id;

        Ok(JsonRpcRequest {
            jsonrpc,
            method,
            params,
            id,
        })
    }

    pub fn with_version(mut self, version: JsonRpcVersion) -> Self {
        self.jsonrpc = Some(version);
        self
    }

    pub fn with_method(mut self, method: String) -> Self {
        self.method = Some(method);
        self
    }

    pub fn with_params<T>(mut self, params: T) -> Self
    where
        T: Serialize,
    {
        self.params = Some(serde_json::to_value(params).unwrap());
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
}

/// A JSONRPC response object. Contains _either_ a `result` (in case of success) or `error` (in case of failure).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct JsonRpcResponse {
    /// JSONRPC version of the response.
    jsonrpc: JsonRpcVersion,
    /// Optional data to be returned in case of success
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    /// Optional data to be returned in case of failure
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
    /// Id corresponding to `id` property of request (if any)
    id: Option<String>,
}

impl JsonRpcResponse {
    pub fn result(&self) -> Option<&Value> {
        match &self.result {
            Some(r) => Some(r),
            None => None,
        }
    }

    /// Deserialize the contained json result (if any)
    pub fn result_as<T>(self) -> Option<T>
    where
        T: DeserializeOwned,
    {
        match self.result {
            Some(r) => Some(serde_json::from_value(r).unwrap()),
            None => None,
        }
    }

    /// Says whether the response indicates a success or an error.
    /// ### Panics
    /// If both `result` and `error` is `Some` or `None`
    pub fn kind(&self) -> ResponseKind {
        match (&self.result, &self.error) {
            (Some(_), Some(_)) => panic!("Result and Error can't both be Some"),
            (Some(result), None) => ResponseKind::Success(result),
            (None, Some(error)) => ResponseKind::Error(error),
            (None, None) => panic!("Result and Error can't both be None"),
        }
    }

    /// Create a `JsonRpcResponse` from a `Result`.
    pub fn from_result<T>(
        jsonrpc: JsonRpcVersion,
        result: Result<T, Error>,
        id: Option<String>,
    ) -> Self
    where
        T: Serialize,
    {
        match result {
            Ok(s) => Self::success(jsonrpc, s, id),
            Err(e) => Self::error(jsonrpc, e, id),
        }
    }

    /// Create a `JsonRpcResponse` with a `result` property (indicating success).
    pub fn success<T: Serialize>(jsonrpc: JsonRpcVersion, result: T, id: Option<String>) -> Self {
        Self {
            jsonrpc,
            result: Some(serde_json::to_value(result).expect("infallible")),
            error: None,
            id,
        }
    }

    /// Create a `JsonRpcResponse` with an `error` property (indicating failure).
    pub fn error(jsonrpc: JsonRpcVersion, error: Error, id: Option<String>) -> Self {
        Self {
            jsonrpc,
            result: None,
            error: Some(error),
            id,
        }
    }

    pub fn get_error(&self) -> Option<&Error> {
        match &self.error {
            None => None,
            Some(err) => Some(&err),
        }
    }
}

pub enum ResponseKind<'a> {
    Success(&'a Value),
    Error(&'a Error),
}

/// Error object to be returned in a `JsonRpcResponse` if something failed.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Error {
    /// JSONRPC error code.
    code: i32,
    /// Short description of what went wrong.
    message: String,
    /// Optional field containing structured error information.
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl Error {
    /// Set the `message` property on `self`.
    pub fn with_message<T>(mut self, message: T) -> Self
    where
        T: Into<String>,
    {
        self.message = message.into();
        self
    }

    /// Set the `data` property on `self`.
    pub fn with_data<T: Serialize>(mut self, data: T) -> Self {
        self.data = Some(serde_json::to_value(data).expect("infallible"));
        self
    }

    /// Constructor for a "Method not found" JSONRPC error.
    pub fn method_not_found() -> Self {
        Self {
            code: ErrorCode::MethodNotFound.into(),
            message: "Method not found".into(),
            data: None,
        }
    }

    /// Constructor for a "Invalid request" JSONRPC error.
    pub fn invalid_request() -> Self {
        Self {
            code: ErrorCode::InvalidRequest.into(),
            message: "Invalid request".into(),
            data: None,
        }
    }

    /// Constructor for an "Invalid params" JSONRPC error.
    pub fn invalid_params() -> Self {
        Self {
            code: ErrorCode::InvalidParams.into(),
            message: "Invalid params".into(),
            data: None,
        }
    }

    /// Constructor for an "Internal error" JSONRPC error.
    pub fn internal_error() -> Self {
        Self {
            code: ErrorCode::InternalError.into(),
            message: "Internal error".into(),
            data: None,
        }
    }

    /// Constructor for an "Invalid format" webserver error.
    pub fn invalid_format(serde_error: serde_json::Error) -> Self {
        Self::invalid_params().with_data(format!("invalid format: '{}'", serde_error))
    }

    /// Constructor for a "Not permitted" webserver error.
    pub fn not_permitted() -> Self {
        Self::internal_error().with_data("not permitted")
    }

    pub fn database_error() -> Self {
        Self::internal_error().with_data("database error")
    }

    /// Constructor for a "Method not implemented" webserver error.
    pub fn not_implemented() -> Self {
        Self::internal_error().with_data("method not implemented")
    }

    /// Constructor for an "Invalid username or password" webserver error.
    pub fn invalid_username_or_password() -> Self {
        Self::invalid_params().with_data("invalid username or passwor")
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

/// Standard JSONRPC error variants as defined by the [JSONRPC specification](https://www.jsonrpc.org/specification#error_object)
pub enum ErrorCode {
    /// Invalid JSON was received.
    ParseError,
    /// The JSON received was not a valid JSONRPC request object.
    InvalidRequest,
    /// The method does not exist / is not available.
    MethodNotFound,
    /// Invalid method parameter(s).
    InvalidParams,
    /// Internal JSONRPC error
    InternalError,
}

impl From<ErrorCode> for i32 {
    fn from(error_code: ErrorCode) -> Self {
        match error_code {
            ErrorCode::ParseError => -32700,
            ErrorCode::InvalidRequest => -32600,
            ErrorCode::MethodNotFound => -32601,
            ErrorCode::InvalidParams => -32602,
            ErrorCode::InternalError => -32603,
        }
    }
}
