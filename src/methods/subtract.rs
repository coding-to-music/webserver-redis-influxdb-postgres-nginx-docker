use super::{Request, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(super) struct SubtractParams {
    a: i32,
    b: i32,
}

pub(super) fn subtract(req: Request) -> Response {
    let params = serde_json::from_value::<SubtractParams>(req.params);
    if let Ok(params) = params {
        let result = params.a - params.b;
        Response {
            jsonrpc: req.jsonrpc,
            result: Some(result.into()),
            error: None,
            id: req.id,
        }
    } else {
        Response::invalid_params(req.id)
    }
}
