use std::sync::Arc;

use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use lib::{app::App, auth::TokenHandler, Webserver};
use structopt::StructOpt;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    let env = std::env::var("WEBSERVER_ENV").unwrap_or_else(|_| "test".to_string());

    let env_file_name = format!("{}.env", env);

    if let Err(e) = dotenv::from_filename(&env_file_name) {
        warn!(
            "environment file not found: {}, error: {}",
            env_file_name, e
        );
    }

    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&lib::get_required_env_var("RUST_LOG"))
        .init();

    let opts = Opts::from_args();
    let jwt_secret = opts.jwt_secret.clone();
    let opts = lib::Opts::from(opts);

    let tokens = TokenHandler::new(jwt_secret);

    let app = Arc::new(App::new(opts.clone(), tokens.clone()).await);

    let webserver = Arc::new(Webserver::new(app, tokens));

    let addr = ([0, 0, 0, 0], opts.port).into();

    let service = make_service_fn(|_| {
        let webserver = webserver.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |request| {
                let webserver = webserver.clone();
                lib::entry_point(webserver, request)
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    info!("starting server on {:?}", addr);
    let _ = server.await;
}

#[derive(StructOpt, Debug, Clone)]
pub struct Opts {
    #[structopt(long, default_value = "3000", env = "WEBSERVER_LISTEN_PORT")]
    port: u16,
    #[structopt(long, env = "WEBSERVER_DATABASE_ADDR")]
    database_addr: String,
    #[structopt(long, env = "WEBSERVER_JWT_SECRET")]
    jwt_secret: String,
    #[structopt(long, env = "WEBSERVER_PUBLISH_REQUEST_LOG")]
    publish_request_log: bool,
    #[structopt(long, env = "WEBSERVER_INFLUX_ADDR")]
    influx_addr: Option<String>,
    #[structopt(long, env = "WEBSERVER_INFLUX_TOKEN")]
    influx_token: Option<String>,
    #[structopt(long, env = "WEBSERVER_INFLUX_ORG")]
    influx_org: Option<String>,
    #[structopt(long, env = "WEBSERVER_RESROBOT_API_KEY")]
    resrobot_api_key: String,
}

impl From<Opts> for lib::Opts {
    fn from(
        Opts {
            port,
            database_addr,
            jwt_secret,
            publish_request_log,
            influx_addr,
            influx_token,
            influx_org,
            resrobot_api_key,
        }: Opts,
    ) -> Self {
        lib::Opts {
            port,
            database_addr,
            jwt_secret,
            publish_request_log,
            influx_addr,
            influx_token,
            influx_org,
            resrobot_api_key,
        }
    }
}
