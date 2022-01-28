use crate::{consts::*, model::Agency};
use redis::{redis::AsyncCommands, RedisPool};
use serde::Serialize;
use std::error::Error;

pub struct Serve {
    redis_pool: redis::RedisPool,
}

impl Serve {
    pub fn new(redis_conn: String) -> Self {
        let redis_pool = RedisPool::new(redis_conn);
        Self { redis_pool }
    }

    #[allow(unreachable_code)]
    pub async fn serve(self) {
        rouille::start_server("localhost:8000", move |request| {
            router!(request,
                (GET) (/agency/{id:String}) => {
                    self.get_agency(&id)
                },

                (GET) (/route/{_id:String}) => {
                    panic!("Oops!")
                },

                (GET) (/{id: u32}) => {
                    println!("u32 {:?}", id);
                    rouille::Response::empty_400()
                },

                (GET) (/{id: String}) => {
                    println!("String {:?}", id);
                    rouille::Response::text(format!("hello, {}", id))
                },

                _ => rouille::Response::empty_404()
            )
        });
    }

    fn get_agency(&self, agency_id: &str) -> rouille::Response {
        // Builds a `Response` object that contains the "hello world" text.
        let agency = futures::executor::block_on(self.get_agency_async(&agency_id));
        match agency {
            Ok(agency) => match agency {
                Some(agency) => json_response(agency),
                None => not_found_response(),
            },
            Err(_err) => internal_server_error_response(None),
        }
    }

    async fn get_agency_async(&self, agency_id: &str) -> Result<Option<Agency>, Box<dyn Error>> {
        let mut conn = self.redis_pool.get_connection().await?;
        let agency: Option<String> = conn.hget(AGENCY_REDIS_KEY, agency_id).await?;

        if let Some(agency) = agency {
            let agency = serde_json::from_str(&agency)?;
            Ok(Some(agency))
        } else {
            Ok(None)
        }
    }
}

fn not_found_response() -> rouille::Response {
    rouille::Response::text("").with_status_code(404)
}

fn json_response<T>(object: T) -> rouille::Response
where
    T: Serialize,
{
    rouille::Response::json(&object).with_status_code(200)
}

fn internal_server_error_response(message: Option<String>) -> rouille::Response {
    rouille::Response::text(message.unwrap_or(String::new())).with_status_code(500)
}
