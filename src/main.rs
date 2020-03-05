#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use actix_web::{middleware, App, HttpServer};
use dotenv::dotenv;

mod app;
mod controllers;

#[actix_rt::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(app::handle_request)
    })
    .bind("127.0.0.1:3000")
    .expect("failed to bind")
    .run()
    .await
    .unwrap()
}

/// Utility method to retrieve an environment variable or panic with the name of the missing variable
pub fn get_env_var(var: &str) -> String {
    std::env::var(var).expect(&format!(r#"missing required env var "{}""#, var))
}
