use actix_web::{middleware, web, App, HttpServer, Responder};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default()) // enable logger
            .service(web::resource("/api/{name}").to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn hello(route: web::Path<(String)>) -> impl Responder {
    format!("Hello {}!\n", route)
}
