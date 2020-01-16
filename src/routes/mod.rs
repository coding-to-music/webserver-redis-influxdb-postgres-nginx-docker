mod goodbye;
mod hello;

pub use goodbye::goodbye;
pub use hello::hello;

pub(crate) type HttpResult = actix_web::Result<actix_web::HttpResponse, actix_web::Error>;
