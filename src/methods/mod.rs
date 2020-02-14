use actix_web::{post, HttpResponse};
use add::add;
use bytes::Bytes;
use futures::future;
use multiply::multiply;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sleep::sleep;
use subtract::subtract;

mod add;
mod multiply;
mod sleep;
mod subtract;

#[derive(Serialize, Deserialize)]
pub enum JsonRpcVersion {
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
    pub fn method_not_found(id: Option<String>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::Two,
            result: None,
            error: Some(Error {
                code: i32::from(ErrorCode::MethodNotFound),
                message: "Method not found".to_string(),
                data: None,
            }),
            id,
        }
    }

    pub fn parse_error() -> Self {
        Self {
            jsonrpc: JsonRpcVersion::Two,
            result: None,
            error: Some(Error {
                code: ErrorCode::ParseError.into(),
                message: "Parse error".into(),
                data: None,
            }),
            id: None,
        }
    }

    pub fn invalid_request(id: Option<String>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::Two,
            result: None,
            error: Some(Error {
                code: ErrorCode::InvalidRequest.into(),
                message: "Invalid request".into(),
                data: None,
            }),
            id,
        }
    }

    pub fn internal_error() -> Self {
        Self {
            jsonrpc: JsonRpcVersion::Two,
            result: None,
            error: Some(Error {
                code: ErrorCode::InternalError.into(),
                message: "Internal error".into(),
                data: None,
            }),
            id: None,
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

#[post("/api")]
pub async fn handle_request(body: Bytes) -> HttpResponse {
    match try_handle(body).await {
        Ok(response) => response,
        Err(e) => {
            error!("{}", e);
            HttpResponse::Ok()
                .content_type("application/json")
                .body(serde_json::to_string(&JsonRpcResponse::internal_error()).unwrap())
        }
    }
}

async fn try_handle(body: Bytes) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let request = serde_json::from_slice(body.as_ref());
    let response_body = if let Ok(request) = request {
        match request {
            Value::Object(_) => {
                let response = handle_single(request).await;
                serde_json::to_string(&response).unwrap()
            }
            Value::Array(values) => {
                let responses = handle_batch(values).await;
                serde_json::to_string(&responses).unwrap()
            }
            _ => serde_json::to_string(&JsonRpcResponse::invalid_request(None)).unwrap(),
        }
    } else {
        serde_json::to_string(&JsonRpcResponse::parse_error()).unwrap()
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(response_body))
}

async fn handle_single(req: Value) -> JsonRpcResponse {
    // extract request id (if any) before deserializing to give better error
    let id: Option<String> = req
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let rpc_request: Result<JsonRpcRequest, _> = serde_json::from_value(req);
    if let Ok(rpc_request) = rpc_request {
        match rpc_request.method.as_str() {
            "add" => add(rpc_request),
            "subtract" => subtract(rpc_request),
            "multiply" => multiply(rpc_request),
            "sleep" => sleep(rpc_request).await,
            _ => JsonRpcResponse::method_not_found(rpc_request.id),
        }
    } else {
        JsonRpcResponse::invalid_request(id)
    }
}

async fn handle_batch(reqs: Vec<Value>) -> Vec<JsonRpcResponse> {
    future::join_all(
        reqs.into_iter()
            .map(|req| handle_single(req))
            .collect::<Vec<_>>(),
    )
    .await
}
