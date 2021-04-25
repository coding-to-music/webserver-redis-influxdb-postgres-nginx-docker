use crate::{JsonRpcRequest, JsonRpcResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum QueueMessage {
    RequestLog {
        request: JsonRpcRequest,
        request_ts_s: i64,
        response: Option<JsonRpcResponse>,
        duration_ms: i64,
    },
}

impl QueueMessage {
    pub fn request_log(
        request: JsonRpcRequest,
        request_ts_s: i64,
        response: Option<JsonRpcResponse>,
        duration_ms: i64,
    ) -> Self {
        QueueMessage::RequestLog {
            request,
            request_ts_s,
            response,
            duration_ms,
        }
    }
}
