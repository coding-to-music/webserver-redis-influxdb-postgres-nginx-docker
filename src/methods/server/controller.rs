use super::*;
use crate::db::{Database, User};
use std::sync::Arc;

pub struct ServerController {
    user_db: Arc<Database<User>>,
}

impl ServerController {
    pub fn new(user_db: Arc<Database<User>>) -> Self {
        Self { user_db }
    }

    pub async fn sleep<T: TryInto<SleepParams, Error = SleepParamsInvalid>>(
        &self,
        request: T,
    ) -> Result<SleepResult, Error> {
        let params: SleepParams = request.try_into()?;

        trace!("Sleeping for {} seconds...", params.seconds());

        let now = std::time::Instant::now();
        tokio::time::delay_for(std::time::Duration::from_secs_f32(params.seconds())).await;
        let elapsed = now.elapsed();

        trace!("Slept for {:?}", elapsed);

        Ok(SleepResult::new(elapsed.as_secs_f32()))
    }
}
