use super::HttpResult;
use crate::Version;
use actix_web::http::{header, Method, StatusCode};
use actix_web::{error, guard, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GoodbyeReq {
    version: Version,
    name: String,
}

pub async fn goodbye(req: web::Path<GoodbyeReq>) -> HttpResult {
    Ok(match req.version {
        Version::V1 => goodbye_v1(req.into_inner()),
        Version::V2 => goodbye_v2(req.into_inner()),
    })
}

fn goodbye_v1(req: GoodbyeReq) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("Goodbye {}!\n", req.name))
}

fn goodbye_v2(req: GoodbyeReq) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(format!("See ya, {}!\n", req.name))
}
