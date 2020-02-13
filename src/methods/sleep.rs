use super::{JsonRpcRequest, JsonRpcResponse};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub(super) struct SleepParams {
    s: u64,
}

pub(super) async fn sleep(req: JsonRpcRequest) -> JsonRpcResponse {
    let params: SleepParams = serde_json::from_value(req.params).unwrap();
    tokio::time::delay_for(Duration::from_secs(params.s)).await;
    JsonRpcResponse {
        jsonrpc: req.jsonrpc,
        result: Some(params.s.into()),
        error: None,
        id: req.id,
    }
}
