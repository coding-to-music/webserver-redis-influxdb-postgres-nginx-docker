use crate::Version;
use serde::Deserialize;
use actix_web::{web, Responder};


#[derive(Debug, Clone, Deserialize)]
pub struct HelloReq {
    version: Version,
    name: String
}


pub async fn hello(req: web::Path<HelloReq>) -> impl Responder {
    match req.version {
        Version::V1 => format!("Hello {}!\n", req.name),
        Version::V2 => format!("Hi {}!\n", req.name)
    }
}

