use add::add;
use multiply::multiply;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use subtract::subtract;
use warp::Reply;
use sleep::sleep;

mod add;
mod multiply;
mod subtract;
mod sleep;

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize)]
pub struct JsonRpcResponse {
    jsonrpc: JsonRpcVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
    id: Option<String>,
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

pub fn handle_request(body: Value) -> impl Reply {
    if body.is_object() {
        warp::reply::json(&handle_single(serde_json::from_value(body).unwrap()))
    } else if let Value::Array(values) = body {
        warp::reply::json(&handle_batch(
            values
                .into_iter()
                .map(|value| serde_json::from_value(value).unwrap())
                .collect(),
        ))
    } else {
        warp::reply::json(&JsonRpcResponse {
            jsonrpc: JsonRpcVersion::Two,
            result: None,
            error: Some(Error {
                code: i32::from(ErrorCode::InvalidRequest),
                message: "Invalid Request".to_string(),
                data: None,
            }),
            id: None,
        })
    }
}

fn handle_single(req: JsonRpcRequest) -> JsonRpcResponse {
    info!("method: {}", req.method);
    match req.method.as_str() {
        "add" => add(req),
        "subtract" => subtract(req),
        "multiply" => multiply(req),
        "sleep" => sleep(req),
        _ => JsonRpcResponse {
            jsonrpc: req.jsonrpc,
            result: None,
            error: Some(Error {
                code: i32::from(ErrorCode::MethodNotFound),
                message: "Method not found".to_string(),
                data: None,
            }),
            id: req.id,
        },
    }
}

fn handle_batch(reqs: Vec<JsonRpcRequest>) -> Vec<JsonRpcResponse> {
    reqs.into_iter().map(|req| handle_single(req)).collect()
}
