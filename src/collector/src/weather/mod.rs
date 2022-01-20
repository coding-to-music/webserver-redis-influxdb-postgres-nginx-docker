use std::{collections::HashMap, time::Duration};

use self::smhi::Root;
use crate::WeatherOpts;
use influxrs::{InfluxClient, Measurement};
use isahc::{AsyncReadResponseExt, HttpClient};

mod smhi;

pub(crate) struct Weather;

impl Weather {
    pub async fn run(&self, opts: WeatherOpts) {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * opts.minute_interval));
        let client = WeatherClient::new(opts);
        loop {
            interval.tick().await;
            client.collect_weather_data().await;
        }
    }
}

struct WeatherClient {
    opts: WeatherOpts,
    influx_client: InfluxClient,
    http_client: HttpClient,
}

impl WeatherClient {
    fn new(opts: WeatherOpts) -> Self {
        let influx_client = InfluxClient::builder(
            opts.influx_url.to_owned(),
            opts.influx_token.to_owned(),
            opts.influx_org.to_owned(),
        )
        .build()
        .unwrap();

        Self {
            opts,
            influx_client,
            http_client: HttpClient::new().unwrap(),
        }
    }

    pub async fn collect_weather_data(&self) {
        let data = self.get_data().await;
        let measurements = create_measurements(data);

        self.influx_client
            .write("weather", &measurements)
            .await
            .unwrap();
    }

    async fn get_data(&self) -> HashMap<String, Root> {
        let mut stations = HashMap::new();
        for station in &self.opts.stations {
            let uri = format!("https://opendata-download-metobs.smhi.se/api/version/1.0/parameter/1/station/{}/period/latest-hour/data.json", station);
            let response: Root = self
                .http_client
                .get_async(&uri)
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            stations.insert(station.clone(), response);
        }
        stations
    }
}

fn create_measurements(stations: HashMap<String, Root>) -> Vec<Measurement> {
    let mut measurements = Vec::new();

    for (_station, root) in stations {
        let value = &root.value[0];

        measurements.push(
            Measurement::builder("air_temp")
                .field("celsius", value.value.parse::<f64>().unwrap())
                .tag("station", root.station.name)
                .timestamp_ms(value.date)
                .build()
                .unwrap(),
        );
    }

    measurements
}
