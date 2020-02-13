use super::{JsonRpcRequest, JsonRpcResponse};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub(super) struct SleepParams {
    ms: u64,
}

pub(super) fn sleep(req: JsonRpcRequest) -> JsonRpcResponse {
    let params: SleepParams = serde_json::from_value(req.params).unwrap();

    thread::sleep(Duration::from_millis(params.ms));
    JsonRpcResponse {
        jsonrpc: req.jsonrpc,
        result: Some(params.ms.into()),
        error: None,
        id: req.id,
    }
}
