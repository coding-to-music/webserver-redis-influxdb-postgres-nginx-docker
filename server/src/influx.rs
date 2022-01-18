use std::time::Instant;

use influxrs::Measurement;

pub struct InfluxClient {
    client: Option<influxrs::InfluxClient>,
}

impl InfluxClient {
    pub fn new(
        url: Option<String>,
        token: Option<String>,
        org: Option<String>,
    ) -> Result<InfluxClient, ()> {
        if let (Some(url), Some(token), Some(org)) = (url, token, org) {
            let client = influxrs::InfluxClient::builder(url, token, org)
                .build()
                .map_err(|_| ())?;
            Ok(Self {
                client: Some(client),
            })
        } else {
            Ok(Self { client: None })
        }
    }

    pub async fn send_request_log(
        &self,
        method: &str,
        duration_ms: i64,
        timestamp_ts_s: i64,
    ) -> Result<(), ()> {
        if let Some(client) = &self.client {
            let timer = Instant::now();
            client
                .write(
                    "server",
                    &[Measurement::builder("request")
                        .tag("method", method)
                        .field("duration_ms", duration_ms)
                        .timestamp(timestamp_ts_s as u128)
                        .build()
                        .unwrap()],
                )
                .await
                .map_err(|_| ())?;
            info!("writing request logs to influx took {:?}", timer.elapsed());
        }
        Ok(())
    }
}
