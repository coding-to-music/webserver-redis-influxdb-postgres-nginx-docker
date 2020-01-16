use actix_web::{middleware, web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize)]
enum Version {
    #[serde(alias="v1")]
    V1,
    #[serde(alias="v2")]
    V2
}

#[derive(Debug, Clone, Deserialize)]
struct HelloReq {
    version: Version,
    name: String
}

#[derive(Debug, Clone, Deserialize)]
struct GoodbyeReq {
    version: Version,
    name: String
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default()) // enable logger
            .service(web::resource("/api/{version}/hello/{name}").to(hello))
            .service(web::resource("/api/{version}/goodbye/{name}").to(goodbye))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn hello(req: web::Path<HelloReq>) -> impl Responder {
    match req.version {
        Version::V1 => format!("Hello {}!\n", req.name),
        Version::V2 => format!("Hi {}!\n", req.name)
    }
}

async fn goodbye(req: web::Path<GoodbyeReq>) -> impl Responder {
    match req.version {
        Version::V1=>format!("Goodbye {}!\n", req.name),
        Version::V2=>format!("See ya, {}!\n", req.name)
    }
}
