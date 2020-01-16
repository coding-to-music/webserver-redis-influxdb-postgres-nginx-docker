use crate::Version;
use actix_web::{web, Responder};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GoodbyeReq {
    version: Version,
    name: String,
}

pub async fn goodbye(req: web::Path<GoodbyeReq>) -> impl Responder {
    error!("routed to goodbye");
    match req.version {
        Version::V1 => goodbye_v1(req.into_inner()),
        Version::V2 => goodbye_v2(req.into_inner()),
    }
}

fn goodbye_v1(req: GoodbyeReq) -> String {
    format!("Goodbye {}!\n", req.name)
}

fn goodbye_v2(req: GoodbyeReq) -> String {
    format!("See ya, {}!\n", req.name)
}
