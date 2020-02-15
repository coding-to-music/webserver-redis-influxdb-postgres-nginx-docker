use super::{Error, Request, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub(super) struct SleepParams {
    s: u64,
}

pub(super) async fn sleep(req: Request) -> Response {
    let params = serde_json::from_value::<SleepParams>(req.params);

    if let Ok(params) = params {
        if params.s > 10 {
            Response::error(
                Error::invalid_params()
                    .with_data(r#"property "s" cannot be larger than 10"#)
                    .build(),
                req.id,
            )
        } else {
            tokio::time::delay_for(Duration::from_secs(params.s)).await;
            Response::success(params.s, req.id)
        }
    } else {
        Response::error(Error::invalid_params(), req.id)
    }
}
