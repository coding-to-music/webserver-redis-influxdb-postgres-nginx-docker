#[macro_use]
extern crate log;

use clap::Parser;

mod weather;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Subcommand {
    Weather(WeatherOpts),
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct WeatherOpts {
    #[clap(long, env = "COLLECTOR_INFLUX_URL")]
    influx_url: String,
    #[clap(long, env = "COLLECTOR_INFLUX_TOKEN")]
    influx_token: String,
    #[clap(long, env = "COLLECTOR_INFLUX_ORG")]
    influx_org: String,
    #[clap(long, env = "COLLECTOR_STATIONS")]
    stations: String,
    #[clap(long, default_value = "30")]
    minute_interval: u64,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap_or("info".to_string()))
        .init();
    let cmd = Subcommand::parse();
    match cmd {
        Subcommand::Weather(weather_opts) => weather::Weather.run(weather_opts).await,
    }
}
