use actix_web::{post, HttpResponse};
use bytes::Bytes;
use futures::future;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod add;
mod multiply;
mod sleep;
mod subtract;

#[derive(Serialize, Deserialize)]
pub enum Version {
    #[serde(alias = "2.0", rename = "2.0")]
    Two,
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    jsonrpc: Version,
    method: String,
    params: Value,
    id: Option<String>,
}

#[derive(Serialize)]
pub struct Response {
    jsonrpc: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
    id: Option<String>,
}

impl Response {
    pub fn success<T: Into<Value>>(result: T, id: Option<String>) -> Self {
        Self {
            jsonrpc: Version::Two,
            result: Some(result.into()),
            error: None,
            id,
        }
    }

    pub fn error(error: Error, id: Option<String>) -> Self {
        Self {
            jsonrpc: Version::Two,
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
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code: code.into(),
            message,
            data: None,
        }
    }

    pub fn build(self) -> Self {
        self
    }

    pub fn with_data<T: Into<Value>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
    }

    pub fn method_not_found() -> Self {
        Self::new(ErrorCode::MethodNotFound, "Method not found".into())
    }

    pub fn parse_error() -> Self {
        Self::new(ErrorCode::ParseError, "Parse error".into())
    }

    pub fn invalid_request() -> Self {
        Self::new(ErrorCode::InvalidRequest, "Invalid request".into())
    }

    pub fn internal_error() -> Self {
        Self::new(ErrorCode::InternalError, "Internal error".into())
    }

    pub fn invalid_params() -> Self {
        Self::new(ErrorCode::InvalidParams, "Invalid params".into())
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

#[post("/api")]
pub async fn handle_request(body: Bytes) -> HttpResponse {
    match try_handle(body).await {
        Ok(response) => response,
        Err(e) => {
            error!("{}", e);
            HttpResponse::Ok().content_type("application/json").body(
                serde_json::to_string(&Response::error(Error::internal_error(), None)).unwrap(),
            )
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
            _ => serde_json::to_string(&Response::error(Error::invalid_request(), None)).unwrap(),
        }
    } else {
        serde_json::to_string(&Response::error(Error::parse_error(), None)).unwrap()
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(response_body))
}

async fn handle_single(req: Value) -> Response {
    // extract request id (if any) before deserializing to give better error
    let id: Option<String> = req
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let rpc_request: Result<Request, _> = serde_json::from_value(req);
    if let Ok(rpc_request) = rpc_request {
        match rpc_request.method.as_str() {
            "add" => add::add(rpc_request),
            "subtract" => subtract::subtract(rpc_request),
            "multiply" => multiply::multiply(rpc_request),
            "sleep" => sleep::sleep(rpc_request).await,
            _ => Response::error(Error::method_not_found(), id),
        }
    } else {
        Response::error(Error::invalid_request(), id)
    }
}

async fn handle_batch(reqs: Vec<Value>) -> Vec<Response> {
    future::join_all(
        reqs.into_iter()
            .map(|req| handle_single(req))
            .collect::<Vec<_>>(),
    )
    .await
}
