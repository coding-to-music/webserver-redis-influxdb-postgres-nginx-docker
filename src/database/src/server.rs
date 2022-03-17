use crate::{Database, DatabaseResult, InsertionResult};
use sqlx::{types::time::OffsetDateTime, Executor};

pub type RequestLogDb = Database<RequestLog>;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
#[non_exhaustive]
pub struct RequestLog {
    id: String,
    request: Request,
    success: bool,
    error_context: Option<String>,
    duration_ms: i64,
    created: OffsetDateTime,
}

impl RequestLog {
    pub fn new(
        id: String,
        request: Request,
        success: bool,
        error_context: Option<String>,
        duration_ms: i64,
        created: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            request,
            success,
            error_context,
            duration_ms,
            created,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Request {
    id: Option<String>,
    method: String,
    timestamp: OffsetDateTime,
}

impl Request {
    pub fn new(id: Option<String>, method: String, timestamp_ms: i64) -> Self {
        let timestamp = OffsetDateTime::from_unix_timestamp(timestamp_ms);
        Self {
            id,
            method,
            timestamp,
        }
    }
}

impl RequestLogDb {
    pub async fn insert_log(
        &self,
        id: &str,
        request: &Request,
        success: bool,
        error_context: &Option<String>,
        duration_ms: i64,
    ) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query(
            "
        INSERT INTO request_log (id, 
            request_id, 
            request_method, 
            request_ts, 
            success, 
            response_error_context,
            duration_ms) 
        VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(id)
        .bind(&request.id)
        .bind(&request.method)
        .bind(request.timestamp)
        .bind(success)
        .bind(error_context)
        .bind(duration_ms)
        .execute(&mut db)
        .await?;

        Ok(InsertionResult::from_changed_rows(
            query_result.rows_affected(),
        ))
    }
}
