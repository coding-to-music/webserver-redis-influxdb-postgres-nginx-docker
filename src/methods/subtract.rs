use super::{JsonRpcRequest, JsonRpcResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(super) struct SubtractParams {
    a: i32,
    b: i32,
}

pub(super) fn subtract(req: JsonRpcRequest) -> JsonRpcResponse {
    let params: SubtractParams = serde_json::from_value(req.params).unwrap();
    let result = params.a - params.b;
    JsonRpcResponse {
        jsonrpc: req.jsonrpc,
        result: Some(result.into()),
        error: None,
        id: req.id,
    }
}
