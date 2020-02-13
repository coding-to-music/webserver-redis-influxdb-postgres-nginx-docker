use super::{JsonRpcRequest, JsonRpcResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(super) struct AddParams {
    a: i32,
    b: i32,
}

pub(super) fn add(req: JsonRpcRequest) -> JsonRpcResponse {
    let params: AddParams = serde_json::from_value(req.params).unwrap();
    let result = params.a + params.b;
    JsonRpcResponse {
        jsonrpc: req.jsonrpc,
        result: Some(result.into()),
        error: None,
        id: req.id,
    }
}
