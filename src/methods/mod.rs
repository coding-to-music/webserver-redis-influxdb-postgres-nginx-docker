pub use prediction::PredictionController;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use sleep::SleepController;
use std::{convert::Infallible, fmt::Debug, str::FromStr};
pub use user::{User, UserController};

mod prediction;
mod sleep;
mod user;

#[derive(Serialize, Deserialize, Clone)]
pub enum JsonRpcVersion {
    #[serde(alias = "1.0", rename = "1.0")]
    One,
    #[serde(alias = "2.0", rename = "2.0")]
    Two,
}

/// A JSONRPC request.
#[derive(Serialize, Deserialize)]
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

/// A JSONRPC response object. Contains _either_ a `result` (in case of success) or `error` (in case of failure) property.
#[derive(Serialize)]
pub struct JsonRpcResponse {
    /// JSONRPC version of the response.
    jsonrpc: JsonRpcVersion,
    /// Optional structured data to be returned in case of success
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    /// Optional data to be returned in case of failure
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
    /// Id corresponding to `id` property of request (if any)
    id: Option<String>,
}

impl JsonRpcResponse {
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

/// Error object to be returned in a `JsonRpcResponse` if something failed.
#[derive(Serialize)]
pub struct Error {
    /// JSONRPC error code.
    code: i32,
    /// Short description of what went wrong.
    message: String,
    /// Optional field containing structured error information.
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    /// Contains debug information about what caused an error.
    /// This property is not exposed to callers of the api.
    #[serde(skip_serializing)]
    internal_data: Option<String>,
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

    /// Set the `internal_data` property on `self`.
    pub fn with_internal_data<T: Debug>(mut self, data: T) -> Self {
        self.internal_data = Some(format!("{:?}", data));
        self
    }

    /// Constructor for a "Method not found" JSONRPC error.
    pub fn method_not_found() -> Self {
        Self {
            code: ErrorCode::MethodNotFound.into(),
            message: "Method not found".into(),
            data: None,
            internal_data: None,
        }
    }

    /// Constructor for a "Invalid request" JSONRPC error.
    pub fn invalid_request() -> Self {
        Self {
            code: ErrorCode::InvalidRequest.into(),
            message: "Invalid request".into(),
            data: None,
            internal_data: None,
        }
    }

    /// Constructor for an "Invalid params" JSONRPC error.
    pub fn invalid_params() -> Self {
        Self {
            code: ErrorCode::InvalidParams.into(),
            message: "Invalid params".into(),
            data: None,
            internal_data: None,
        }
    }

    /// Constructor for an "Internal error" JSONRPC error.
    pub fn internal_error() -> Self {
        Self {
            code: ErrorCode::InternalError.into(),
            message: "Internal error".into(),
            data: None,
            internal_data: None,
        }
    }

    pub fn get_internal_data(&self) -> Option<&str> {
        match &self.internal_data {
            None => None,
            Some(data) => Some(data),
        }
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::internal_error().with_internal_data(e)
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
}

impl FromStr for Method {
    type Err = (); // any failure means the method simply doesn't exist
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add_prediction" => Ok(Self::AddPrediction),
            "delete_prediction" => Ok(Self::DeletePrediction),
            "search_predictions" => Ok(Self::SearchPredictions),
            "add_user" => Ok(Self::AddUser),
            "change_password" => Ok(Self::ChangePassword),
            "validate_user" => Ok(Self::ValidateUser),
            "sleep" => Ok(Self::Sleep),
            _ => Err(()),
        }
    }
}
