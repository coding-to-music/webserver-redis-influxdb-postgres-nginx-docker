use super::HttpResult;
use crate::Version;
use actix_web::http::{header, Method, StatusCode};
use actix_web::{error, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HelloReq {
    version: Version,
    name: String,
}

pub async fn hello(req: web::Path<HelloReq>) -> HttpResult {
    Ok(match req.version {
        Version::V1 => hello_v1(req.into_inner()),
        Version::V2 => hello_v2(req.into_inner()),
    })
}

fn hello_v1(req: HelloReq) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("Hello {}!\n", req.name))
}

fn hello_v2(req: HelloReq) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("Hi {}!\n", req.name))
}
