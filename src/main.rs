use actix_web::{middleware, web, App, HttpServer, Responder};
use serde::{Deserialize};

mod routes;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Version {
    #[serde(alias="v1")]
    V1,
    #[serde(alias="v2")]
    V2
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default()) // enable logger
            .service(web::resource("/api/{version}/hello/{name}").to(routes::hello))
            .service(web::resource("/api/{version}/goodbye/{name}").to(routes::goodbye))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
