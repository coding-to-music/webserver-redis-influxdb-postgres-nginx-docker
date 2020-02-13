use add::add;
use futures::future;
use multiply::multiply;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sleep::sleep;
use std::convert::Infallible;
use subtract::subtract;
use warp::reject::Rejection;
use warp::Reply;
use warp::filters::body::BodyDeserializeError;

mod add;
mod multiply;
mod sleep;
mod subtract;

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

pub async fn handle_request(body: Value) -> Result<impl Reply, Infallible> {
    if body.is_object() {
        Ok(warp::reply::json(
            &handle_single(serde_json::from_value(body).unwrap()).await,
        ))
    } else if let Value::Array(values) = body {
        Ok(warp::reply::json(
            &handle_batch(
                values
                    .into_iter()
                    .map(|value| serde_json::from_value(value).unwrap())
                    .collect(),
            )
            .await,
        ))
    } else {
        Ok(warp::reply::json(&JsonRpcResponse {
            jsonrpc: JsonRpcVersion::Two,
            result: None,
            error: Some(Error {
                code: i32::from(ErrorCode::InvalidRequest),
                message: "Invalid Request".to_string(),
                data: None,
            }),
            id: None,
        }))
    }
}

async fn handle_single(req: JsonRpcRequest) -> JsonRpcResponse {
    info!("method: {}", req.method);
    match req.method.as_str() {
        "add" => add(req),
        "subtract" => subtract(req),
        "multiply" => multiply(req),
        "sleep" => sleep(req).await,
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

async fn handle_batch(reqs: Vec<JsonRpcRequest>) -> Vec<JsonRpcResponse> {
    future::join_all(
        reqs.into_iter()
            .map(|req| handle_single(req))
            .collect::<Vec<_>>(),
    )
    .await
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if let Some(_) = err.find::<BodyDeserializeError>() {
        (i32::from(ErrorCode::InvalidRequest), "Invalid Request")
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        (-32000, "Server Error")
    };

    let json = warp::reply::json(&JsonRpcResponse {
        jsonrpc: JsonRpcVersion::Two,
        result: None,
        error: Some(Error {
            code: code,
            message: message.into(),
            data: None,
        }),
        id: None,
    });

    Ok(json)
}
