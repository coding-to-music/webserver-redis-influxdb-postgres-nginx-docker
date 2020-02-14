#[macro_use]
extern crate log;

use actix_web::{middleware, App, HttpServer};
use dotenv::dotenv;

mod methods;

#[actix_rt::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(methods::handle_request)
    })
    .bind("127.0.0.1:3030")
    .unwrap()
    .run()
    .await
    .unwrap()
}
