use crate::app::{AppResult, ParamsError};
use contracts::{sas, server, JsonRpcRequest};
use hmac::{Hmac, Mac, NewMac};
use server::sleep;
use sha2::Sha256;
use std::{convert::TryFrom, time};

pub struct ServerController {}

impl ServerController {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn sleep(&self, request: JsonRpcRequest) -> AppResult<sleep::MethodResult> {
        use sleep::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let timer = time::Instant::now();
        tokio::time::sleep(time::Duration::from_millis(params.ms)).await;
        let elapsed = timer.elapsed();

        Ok(MethodResult::new(elapsed.as_millis() as u64))
    }

    pub async fn generate_sas_key(&self, request: JsonRpcRequest) -> AppResult<sas::MethodResult> {
        use sas::{MethodResult, Params};
        let params = Params::try_from(request)?;

        let token = Self::generate(
            unix_now(),
            params.weeks_expiry,
            &params.resource_uri,
            &params.key_value,
            &params.key_name,
        )?;

        Ok(MethodResult::new(token))
    }

    fn generate(
        now: i64,
        weeks_expiry: u32,
        resource_uri: &str,
        key_value: &str,
        key_name: &str,
    ) -> AppResult<String> {
        type HmacSha256 = Hmac<Sha256>;
        const SECONDS_IN_ONE_WEEK: i64 = 60 * 60 * 24 * 7;
        let expiry = now + weeks_expiry as i64 * SECONDS_IN_ONE_WEEK;
        let sign = format!("{}\n{}", urlencoding::encode(&resource_uri), expiry);
        let mut hmac = HmacSha256::new_from_slice(key_value.as_bytes())?;
        hmac.update(sign.as_bytes());
        let result = hmac.finalize().into_bytes();
        let signature = base64::encode(result);
        let token = format!(
            "SharedAccessSignature sr={}&sig={}&se={}&skn={}",
            urlencoding::encode(&resource_uri),
            urlencoding::encode(&signature),
            expiry,
            key_name
        );
        Ok(token)
    }
}

impl ParamsError for sleep::InvalidParams {}
impl ParamsError for sas::InvalidParams {}

fn unix_now() -> i64 {
    chrono::Utc::now().timestamp()
}
