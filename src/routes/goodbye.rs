use crate::Version;
use serde::Deserialize;
use actix_web::{web, Responder};


#[derive(Debug, Clone, Deserialize)]
pub struct GoodbyeReq {
    version: Version,
    name: String
}

pub async fn goodbye(req: web::Path<GoodbyeReq>) -> impl Responder {
    match req.version {
        Version::V1=>format!("Goodbye {}!\n", req.name),
        Version::V2=>format!("See ya, {}!\n", req.name)
    }
}
