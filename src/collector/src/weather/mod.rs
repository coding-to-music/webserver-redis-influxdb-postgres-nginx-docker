use self::data::DataResponse;
use crate::{weather::station::StationResponse, WeatherOpts};
use influxrs::{InfluxClient, Measurement};
use isahc::{AsyncReadResponseExt, HttpClient};
use std::{collections::HashMap, time::Duration};

mod data;
mod station;

pub(crate) struct Weather;

impl Weather {
    pub async fn run(&self, opts: WeatherOpts) {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * opts.minute_interval));
        let client = WeatherClient::new(opts);
        loop {
            interval.tick().await;
            let result = client.collect_weather_data().await;
            match result {
                Ok(_) => info!("successfully collected weather data"),
                Err(e) => error!("failed to collect weather data with error: '{}'", e),
            }
        }
    }
}

#[allow(unused)]
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

    pub async fn collect_weather_data(&self) -> Result<(), String> {
        let data = self.get_data(1).await?;
        let measurements = create_measurements(data)?;

        self.influx_client
            .write("weather", &measurements)
            .await
            .map_err(|influx_err| format!("{}", influx_err))
    }

    async fn get_active_stations(&self, parameter: u32) -> Result<HashMap<String, String>, String> {
        let uri = format!(
            "https://opendata-download-metobs.smhi.se/api/version/1.0/parameter/{}.json",
            parameter
        );
        let response: StationResponse = self
            .http_client
            .get_async(&uri)
            .await
            .map_err(|e| format!("error: {}", e))?
            .json()
            .await
            .map_err(|e| format!("error: {}", e))?;
        Ok(response
            .station
            .into_iter()
            .filter(|s| s.active)
            .map(|s| (s.key, s.name))
            .collect())
    }

    async fn get_data(&self, parameter: u32) -> Result<HashMap<String, DataResponse>, String> {
        let active_stations = self.get_active_stations(parameter).await?;
        let mut station_responses = HashMap::new();
        for (id, _name) in active_stations {
            match self.get_data_for_station(&id).await {
                Ok(response) => {
                    station_responses.insert(id.to_owned(), response);
                }
                Err(e) => warn!("failed to get data for station {} with error: '{}'", id, e),
            }
        }
        Ok(station_responses)
    }

    async fn get_data_for_station(&self, station: &str) -> Result<DataResponse, String> {
        let uri = format!("https://opendata-download-metobs.smhi.se/api/version/1.0/parameter/1/station/{}/period/latest-hour/data.json", station);
        let response: DataResponse = self
            .http_client
            .get_async(&uri)
            .await
            .map_err(|e| format!("error: {}", e))?
            .json()
            .await
            .map_err(|e| format!("error: {}", e))?;
        Ok(response)
    }
}

fn create_measurements(
    stations: HashMap<String, DataResponse>,
) -> Result<Vec<Measurement>, String> {
    let mut measurements = Vec::new();

    for (_station, root) in stations {
        let value = &root.value[0];

        let mut builder = Measurement::builder("air_temp")
            .field("celsius", value.value.parse::<f64>().unwrap())
            .tag("station", root.station.name)
            .timestamp_ms(value.date);
        if let Some(pos) = root.position.get(0) {
            builder = builder
                .field("station_lat", pos.latitude)
                .field("station_lon", pos.longitude);
        }
        measurements.push(
            builder
                .build()
                .map_err(|i| format!("error building measurement: '{}'", i))?,
        );
    }

    Ok(measurements)
}
