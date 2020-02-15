use super::{Request, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(super) struct MultiplyParams {
    a: i32,
    b: i32,
}

pub(super) fn multiply(req: Request) -> Response {
    let params: MultiplyParams = serde_json::from_value(req.params).unwrap();
    let result = params.a * params.b;
    Response {
        jsonrpc: req.jsonrpc,
        result: Some(result.into()),
        error: None,
        id: req.id,
    }
}
