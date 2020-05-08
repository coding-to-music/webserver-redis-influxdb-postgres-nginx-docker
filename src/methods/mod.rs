pub use bookmark::BookmarkController;
pub use prediction::PredictionController;
pub use sleep::SleepController;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{convert::Infallible, str::FromStr};

mod bookmark;
mod prediction;
mod sleep;

#[derive(Serialize, Deserialize, Clone)]
pub enum JsonRpcVersion {
    #[serde(alias = "1.0", rename = "1.0")]
    One,
    #[serde(alias = "2.0", rename = "2.0")]
    Two,
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcRequest {
    jsonrpc: JsonRpcVersion,
    method: String,
    params: Value,
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

#[derive(Serialize)]
pub struct JsonRpcResponse {
    jsonrpc: JsonRpcVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
    id: Option<String>,
}

impl JsonRpcResponse {
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

    pub fn success<T: Serialize>(jsonrpc: JsonRpcVersion, result: T, id: Option<String>) -> Self {
        Self {
            jsonrpc,
            result: Some(serde_json::to_value(result).expect("infallible")),
            error: None,
            id,
        }
    }

    pub fn error(jsonrpc: JsonRpcVersion, error: Error, id: Option<String>) -> Self {
        Self {
            jsonrpc,
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[derive(Serialize)]
pub struct Error {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl Error {
    #[allow(dead_code)]
    pub fn new(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_data<T: Serialize>(mut self, data: T) -> Self {
        self.data = Some(serde_json::to_value(data).expect("infallible"));
        self
    }

    pub fn method_not_found() -> Self {
        Self {
            code: ErrorCode::MethodNotFound.into(),
            message: "Method not found".into(),
            data: None,
        }
    }

    pub fn invalid_request() -> Self {
        Self {
            code: ErrorCode::InvalidRequest.into(),
            message: "Invalid request".into(),
            data: None,
        }
    }

    pub fn invalid_params() -> Self {
        Self {
            code: ErrorCode::InvalidParams.into(),
            message: "Invalid params".into(),
            data: None,
        }
    }

    pub fn internal_error() -> Self {
        Self {
            code: ErrorCode::InternalError.into(),
            message: "Internal error".into(),
            data: None,
        }
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::internal_error()
    }
}

pub enum ErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
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
    SearchBookmark,
    AddBookmark,
    DeleteBookmark,
    AddPrediction,
    Sleep
}

impl FromStr for Method {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "search_bookmark" => Ok(Self::SearchBookmark),
            "add_bookmark" => Ok(Self::AddBookmark),
            "delete_bookmark" => Ok(Self::DeleteBookmark),
            "add_prediction" => Ok(Self::AddPrediction),
            "sleep" => Ok(Self::Sleep),
            _ => Err(()),
        }
    }
}

trait Database {
    fn get_connection(&self) -> Result<rusqlite::Connection, Error>;
}
