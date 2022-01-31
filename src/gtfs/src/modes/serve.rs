use crate::{consts::*, model::Agency};
use redis::{sync_pool::r2d2_redis::redis::Commands, sync_pool::SyncRedisPool as RedisPool};
use serde::Serialize;

pub struct Serve {
    redis_pool: RedisPool,
    listen_port: u32,
}

impl Serve {
    pub fn new(redis_conn: String, listen_port: u32) -> Self {
        let redis_pool = RedisPool::new(redis_conn, 10);
        Self {
            redis_pool,
            listen_port,
        }
    }

    #[allow(unreachable_code)]
    pub async fn serve(self) {
        rouille::start_server(format!("localhost:{}", self.listen_port), move |request| {
            router!(request,
                (GET) (/agency/{agency_id:String}) => {
                    self.get_agency(&agency_id)
                },

                (GET) (/route/{route_id:String}) => {
                    self.get_route(route_id)
                },
                _ => not_found_response()
            )
        });
    }

    fn get_agency(&self, agency_id: &str) -> rouille::Response {
        let mut conn = self.redis_pool.get_connection().unwrap();
        let agency: Option<String> = conn.hget(AGENCY_REDIS_KEY, agency_id).unwrap();

        match agency {
            Some(agency) => {
                let agency: Agency = serde_json::from_str(&agency).unwrap();
                json_response(agency)
            }
            None => not_found_response(),
        }
    }

    fn get_route(&self, route_id: &str) -> rouille::Response {
        let mut conn = self.redis_pool.get_connection().unwrap();
        let route: Option<String> = conn.hget(ROUTE_REDIS_KEY, route_id).unwrap();
        match route {
            Some(route) => {
                let route: Route = serde_json::from_str(&route).unwrap();
                json_response(route)
            }
            None => not_found_response(),
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
