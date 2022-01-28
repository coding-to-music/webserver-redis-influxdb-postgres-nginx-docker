use crate::{consts::*, model::Agency};
use redis::{pool::SyncRedisPool as RedisPool, redis::Commands};
use serde::Serialize;

pub struct Serve {
    redis_pool: RedisPool,
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
        let mut conn = self.redis_pool.get_connection();
        let agency: Option<String> = conn.hget(AGENCY_REDIS_KEY, agency_id).unwrap();

        if let Some(agency) = agency {
            let agency: Agency = serde_json::from_str(&agency).unwrap();
            json_response(agency)
        } else {
            not_found_response()
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
