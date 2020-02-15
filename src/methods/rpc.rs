use super::{JsonRpcRequest, JsonRpcResponse};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub(super) async fn get_methods(req: JsonRpcRequest) -> JsonRpcResponse {
    tokio::time::delay_for(Duration::from_secs(params.s)).await;
    JsonRpcResponse {
        jsonrpc: req.jsonrpc,
        result: Some(params.s.into()),
        error: None,
        id: req.id,
    }
}
