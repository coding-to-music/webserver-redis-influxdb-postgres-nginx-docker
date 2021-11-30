use crate::{Database, DatabaseResult, InsertionResult};
use chrono::{DateTime, TimeZone, Utc};
use sqlx::Executor;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
#[non_exhaustive]
pub struct RequestLog {
    id: String,
    request: Request,
    response: Response,
    error_context: Option<String>,
    duration_ms: i64,
    created_s: i64,
}

impl RequestLog {
    pub fn new(
        id: String,
        request: Request,
        response: Response,
        error_context: Option<String>,
        duration_ms: i64,
        created_s: i64,
    ) -> Self {
        Self {
            id,
            request,
            response,
            error_context,
            duration_ms,
            created_s,
        }
    }

    pub fn created_utc(&self) -> DateTime<Utc> {
        chrono::Utc.timestamp(self.created_s, 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Request {
    id: Option<String>,
    method: String,
    params: String,
    ts_s: i64,
}

impl Request {
    pub fn new(id: Option<String>, method: String, params: String, ts_s: i64) -> Self {
        Self {
            id,
            method,
            params,
            ts_s,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Response {
    result: Option<String>,
    error: Option<String>,
}

impl Response {
    pub fn new(result: Option<String>, error: Option<String>) -> Self {
        Self { result, error }
    }

    fn kind(&self) -> ResponseKind {
        match (&self.result, &self.error) {
            (Some(result), None) => ResponseKind::Success(result),
            (None, Some(error)) => ResponseKind::Error(error),
            (None, None) => ResponseKind::None,
            _ => {
                panic!("invalid response: {:#?}", self);
            }
        }
    }
}

enum ResponseKind<'a> {
    Success(&'a String),
    Error(&'a String),
    None,
}

impl Database<RequestLog> {
    pub async fn insert_log(
        &self,
        RequestLog {
            id,
            request,
            response,
            error_context,
            duration_ms,
            created_s,
        }: &RequestLog,
    ) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection().await?;

        let (result, error): (Option<&String>, Option<&String>) = match response.kind() {
            ResponseKind::Success(result) => (Some(result), None),
            ResponseKind::Error(error) => (None, Some(error)),
            ResponseKind::None => (None, None),
        };

        let query_result = sqlx::query(
            "
        INSERT INTO request_log (id, 
            request_id, 
            request_method, 
            request_params, 
            request_ts_s, 
            response_result, 
            response_error, 
            response_error_context,
            duration_ms, 
            created_s) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .bind(id)
        .bind(&request.id)
        .bind(&request.method)
        .bind(&request.params)
        .bind(request.ts_s)
        .bind(result)
        .bind(error)
        .bind(error_context)
        .bind(duration_ms)
        .bind(created_s)
        .execute(&mut db)
        .await?;

        Ok(InsertionResult::from_changed_rows(
            query_result.rows_affected(),
        ))
    }
}
