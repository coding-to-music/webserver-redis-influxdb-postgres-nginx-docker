#![allow(clippy::new_without_default)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{
    convert::{Infallible, TryFrom},
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
};

pub use methods::*;

mod methods;

mod method_names {
    pub const ADD_LIST_ITEM: &str = "add_list_item";
    pub const GET_LIST_ITEMS: &str = "get_list_items";
    pub const DELETE_LIST_ITEM: &str = "delete_list_item";
    pub const GET_LIST_TYPES: &str = "get_list_types";
    pub const RENAME_LIST_TYPE: &str = "rename_list_type";

    pub const GET_DEPARTURES: &str = "get_departures";

    pub const SLEEP: &str = "sleep";

    pub const ADD_USER: &str = "add_user";
    pub const GET_USER: &str = "get_user";
    pub const GET_TOKEN: &str = "get_token";

    pub const GENERATE_SAS_KEY: &str = "generate_sas_key";
}

pub mod error_codes {
    pub mod standard {
        pub const PARSE_ERROR: i32 = -32700;
        pub const INVALID_REQUEST: i32 = -32600;
        pub const METHOD_NOT_FOUND: i32 = -32601;
        pub const INVALID_PARAMS: i32 = -32602;
        pub const INTERNAL_ERROR: i32 = -32603;
    }

    pub mod application {
        pub const ITEM_DOES_NOT_EXIST: i32 = -31999;
        pub const NOT_AUTHORIZED: i32 = -31998;
    }
}

/// A JSONRPC method
#[derive(Debug, Clone, Copy)]
pub enum Method {
    /// Add a list item
    AddListItem,
    /// Get all list items of a given list type
    GetListItems,
    /// Delete a list item
    DeleteListItem,
    /// Get all existing list types
    GetListTypes,
    /// Rename a list type
    RenameListType,

    /// Get upcoming departures for a given stop
    GetDepartures,

    /// Tell the server to sleep
    Sleep,

    /// Add a user
    AddUser,
    /// Get a user
    GetUser,
    /// Get a JWT
    GetToken,

    /// Generate an SAS key
    GenerateSasKey,
}

impl FromStr for Method {
    type Err = (); // any failure means the method simply doesn't exist
    fn from_str(s: &str) -> Result<Method, Self::Err> {
        use method_names::*;
        use Method::*;
        match s {
            ADD_LIST_ITEM => Ok(AddListItem),
            GET_LIST_ITEMS => Ok(GetListItems),
            DELETE_LIST_ITEM => Ok(DeleteListItem),
            GET_LIST_TYPES => Ok(GetListTypes),
            RENAME_LIST_TYPE => Ok(RenameListType),
            GET_DEPARTURES => Ok(GetDepartures),
            SLEEP => Ok(Sleep),
            GENERATE_SAS_KEY => Ok(GenerateSasKey),
            ADD_USER => Ok(AddUser),
            GET_USER => Ok(GetUser),
            GET_TOKEN => Ok(GetToken),
            _ => Err(()),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use method_names::*;
        use Method::*;
        let ouput = match self {
            AddListItem => ADD_LIST_ITEM,
            GetListItems => GET_LIST_ITEMS,
            DeleteListItem => DELETE_LIST_ITEM,
            GetListTypes => GET_LIST_TYPES,
            RenameListType => RENAME_LIST_TYPE,
            Sleep => SLEEP,
            GetDepartures => GET_DEPARTURES,
            GenerateSasKey => GENERATE_SAS_KEY,
            AddUser => ADD_USER,
            GetUser => GET_USER,
            GetToken => GET_TOKEN,
        };
        write!(f, "{}", ouput)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Copy)]
pub enum JsonRpcVersion {
    #[serde(alias = "2.0", rename = "2.0")]
    Two,
}

/// A JSONRPC request.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[non_exhaustive]
pub struct JsonRpcRequest {
    /// JSONRPC version.
    pub jsonrpc: JsonRpcVersion,
    /// RPC method to call.
    pub method: String,
    /// Parameters to pass to the method.
    pub params: Value,
    /// A response to this request should contain this same id (provided by the requester).
    /// If the request is a notification, then `id` is `None`.
    pub id: Option<String>,
}

impl JsonRpcRequest {
    pub fn new<T>(method: String, params: T, id: Option<String>) -> Self
    where
        T: Serialize,
    {
        Self {
            jsonrpc: JsonRpcVersion::Two,
            method,
            params: serde_json::to_value(params).unwrap(),
            id,
        }
    }

    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

#[derive(Debug)]
pub struct JsonRpcRequestBuilder {
    jsonrpc: Option<JsonRpcVersion>,
    method: Option<String>,
    params: Option<Value>,
    id: Option<String>,
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

#[derive(Debug)]
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

impl Error for JsonRpcRequestBuilderError {}

/// A JSONRPC response object. Contains _either_ a `result` (in case of success) or `error` (in case of failure).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct JsonRpcResponse {
    /// JSONRPC version of the response.
    pub jsonrpc: JsonRpcVersion,
    /// Optional data to be returned in case of success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Optional data to be returned in case of failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Id corresponding to `id` property of request (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl JsonRpcResponse {
    pub fn is_success(&self) -> bool {
        match (&self.result, &self.error) {
            (None, Some(_)) => false,
            (Some(_), None) => true,
            _ => unreachable!()
        }
    }

    pub fn result(&self) -> Option<&Value> {
        match &self.result {
            Some(r) => Some(r),
            None => None,
        }
    }

    /// Deserialize the contained json result (if any)
    pub fn result_as<T>(self) -> Result<Option<T>, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        Ok(match self.result {
            Some(r) => {
                let deserialized: T = serde_json::from_value(r)?;
                Some(deserialized)
            }
            None => None,
        })
    }

    /// Says whether the response indicates a success or an error.
    ///
    /// ## Panics
    /// * If both `result` and `error` is `Some` or `None`
    pub fn kind(&self) -> ResponseKind {
        match (&self.result, &self.error) {
            (Some(_), Some(_)) => panic!("Result and Error can't both be Some"),
            (Some(result), None) => ResponseKind::Success(result),
            (None, Some(error)) => ResponseKind::Error(error),
            (None, None) => panic!("Result and Error can't both be None"),
        }
    }

    /// Create a `JsonRpcResponse` from a `Result`.
    pub fn from_result<T>(result: Result<T, JsonRpcError>, id: Option<String>) -> Self
    where
        T: Serialize,
    {
        match result {
            Ok(s) => Self::success(s, id),
            Err(e) => Self::error(e, id),
        }
    }

    /// Create a `JsonRpcResponse` with a `result` property (indicating success).
    pub fn success<T: Serialize>(result: T, id: Option<String>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::Two,
            result: Some(serde_json::to_value(result).expect("infallible")),
            error: None,
            id,
        }
    }

    /// Create a `JsonRpcResponse` with an `error` property (indicating failure).
    pub fn error(error: JsonRpcError, id: Option<String>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::Two,
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[derive(Debug)]
pub enum ResponseKind<'a> {
    Success(&'a Value),
    Error(&'a JsonRpcError),
}

/// Error object to be returned in a `JsonRpcResponse` if something failed.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct JsonRpcError {
    /// JSONRPC error code.
    pub code: i32,
    /// Short description of what went wrong.
    pub message: String,
    /// Optional field containing structured error information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    fn new(code: ErrorCode, message: String, data: Option<Value>) -> Self {
        Self {
            code: code.into(),
            message,
            data,
        }
    }

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
    ///
    /// ## Definition
    /// The method does not exist / is not available.
    pub fn method_not_found() -> Self {
        Self::new(
            ErrorCode::MethodNotFound,
            "Method not found".to_owned(),
            None,
        )
    }

    /// Constructor for a "Invalid request" JSONRPC error.
    ///
    /// ## Definition
    /// The JSON sent is not a valid Request object.
    pub fn invalid_request() -> Self {
        Self::new(
            ErrorCode::InvalidRequest,
            "Invalid request".to_owned(),
            None,
        )
    }

    /// Constructor for an "Invalid params" JSONRPC error.
    ///
    /// ## Definition
    /// Invalid method parameter(s).
    pub fn invalid_params() -> Self {
        Self::new(ErrorCode::InvalidParams, "Invalid params".to_owned(), None)
    }

    /// Constructor for an "Internal error" JSONRPC error.
    ///
    /// ## Definition
    /// Internal JSON-RPC error.
    pub fn internal_error() -> Self {
        Self::new(ErrorCode::InternalError, "Internal error".to_owned(), None)
    }

    /// Constructor for an "Invalid format" webserver error.
    ///
    /// ## Definition
    /// Invalid JSON was received by the server.
    /// An error occurred on the server while parsing the JSON text.
    pub fn invalid_format(serde_error: serde_json::Error) -> Self {
        Self::invalid_request().with_message(format!("invalid format: '{}'", serde_error))
    }

    /// Constructor for an "Application error" webserver error.
    ///
    /// This error means your request could not be processed due to a failure in the application level logic.
    ///
    /// # Panics
    /// If `code` is a reserved error code according to the JSON-RPC specs. See the [JSONRPC specification](https://www.jsonrpc.org/specification#error_object) for more information.
    pub fn application_error(code: i32) -> Self {
        if ErrorCode::is_reserved(code) {
            panic!("error code '{}' is reserved by the JSON-RPC spec", code);
        }
        Self {
            code,
            message: String::new(),
            data: None,
        }
    }

    /// Constructor for a "Not permitted" webserver error.
    pub fn not_permitted() -> Self {
        Self::internal_error().with_message("not permitted")
    }

    pub fn database_error() -> Self {
        Self::internal_error().with_message("database error")
    }

    /// Constructor for a "Method not implemented" webserver error.
    pub fn not_implemented() -> Self {
        Self::internal_error().with_message("method not implemented")
    }
}

impl From<Infallible> for JsonRpcError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

/// Code identifying which type of error has occurred.
pub enum ErrorCode {
    /// Invalid JSON was received.
    ParseError,
    /// The JSON received was not a valid JSONRPC request object.
    InvalidRequest,
    /// The method does not exist / is not available.
    MethodNotFound,
    /// Invalid method parameter(s).
    InvalidParams,
    /// Internal JSONRPC error.
    InternalError,
}

impl ErrorCode {
    pub fn is_reserved(code: i32) -> bool {
        (-32768..=-32000).contains(&code)
    }
}

impl From<ErrorCode> for i32 {
    fn from(error_code: ErrorCode) -> Self {
        match error_code {
            ErrorCode::ParseError => error_codes::standard::PARSE_ERROR,
            ErrorCode::InvalidRequest => error_codes::standard::INVALID_REQUEST,
            ErrorCode::MethodNotFound => error_codes::standard::METHOD_NOT_FOUND,
            ErrorCode::InvalidParams => error_codes::standard::INVALID_PARAMS,
            ErrorCode::InternalError => error_codes::standard::INTERNAL_ERROR,
        }
    }
}

impl TryFrom<i32> for ErrorCode {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            error_codes::standard::PARSE_ERROR => ErrorCode::ParseError,
            error_codes::standard::INVALID_REQUEST => ErrorCode::InvalidRequest,
            error_codes::standard::INTERNAL_ERROR => ErrorCode::InternalError,
            error_codes::standard::INVALID_PARAMS => ErrorCode::InvalidParams,
            error_codes::standard::METHOD_NOT_FOUND => ErrorCode::MethodNotFound,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct GetTokenRequest {
    pub key_name: String,
    pub key_value: String,
}

impl GetTokenRequest {
    pub fn new(key_name: String, key_value: String) -> Self {
        Self {
            key_name,
            key_value,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct GetTokenResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl GetTokenResponse {
    pub fn success(access_token: String) -> Self {
        Self {
            success: true,
            access_token: Some(access_token),
            error_message: None,
        }
    }

    pub fn error(error_message: String) -> Self {
        Self {
            success: false,
            access_token: None,
            error_message: Some(error_message),
        }
    }
}

fn invalid_params_serde_message(err: &serde_json::Error) -> String {
    format!("invalid format of params object: '{}'", err)
}

fn generic_invalid_value_message(param_name: &str) -> String {
    format!("invalid value of '{}'", param_name)
}

fn invalid_value_because_message(param_name: &str, clarification: String) -> String {
    format!(
        "{}, {}",
        generic_invalid_value_message(param_name),
        clarification
    )
}
