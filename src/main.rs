use dotenv::dotenv;
use warp::Filter;
#[macro_use]
extern crate log;

mod methods;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let log = warp::log("api");
    let handler = warp::post()
        .and(warp::path("api"))
        .and(warp::body::json())
        .and_then(methods::handle_request)
        .recover(methods::handle_rejection)
        .with(log);

    warp::serve(handler).run(([127, 0, 0, 1], 3030)).await;
}
