use super::*;
use crate::{
    db::{Database, User, UserRole},
    methods,
};
use std::{path::PathBuf, sync::Arc, time};
use time::Duration;

pub struct ServerController {
    user_db: Arc<Database<User>>,
    log_directory: PathBuf,
}

impl ServerController {
    pub fn new(user_db: Arc<Database<User>>, log_directory: PathBuf) -> Self {
        Self {
            user_db,
            log_directory,
        }
    }

    pub async fn sleep(&self, request: crate::JsonRpcRequest) -> Result<SleepResult, crate::Error> {
        let params: SleepParams = request.try_into()?;

        self.validate_user_is_admin(params.user())?;

        trace!("Sleeping for {} seconds...", params.seconds());

        let now = std::time::Instant::now();
        tokio::time::delay_for(Duration::from_secs_f32(params.seconds())).await;
        let elapsed = now.elapsed();

        trace!("Slept for {:?}", elapsed);

        Ok(SleepResult::new(elapsed.as_secs_f32()))
    }

    pub async fn clear_logs(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<ClearLogsResult, crate::Error> {
        let params: ClearLogsParams = request.try_into()?;

        self.validate_user_is_admin(params.user())?;

        let paths = std::fs::read_dir(&self.log_directory)
            .map_err(|e| crate::Error::internal_error().with_internal_data(e))?;

        let log_files: Vec<_> = paths
            .filter_map(|p| p.ok())
            .filter_map(|p| match p.metadata() {
                Ok(metadata) if metadata.is_file() => Some(p),
                _ => None,
            })
            .collect();

        info!(
            "found {} log files in directory '{:?}'",
            log_files.len(),
            self.log_directory
        );

        let mut total_size = 0;
        for f in &log_files {
            let size = f
                .metadata()
                .map_err(|e| crate::Error::internal_error().with_internal_data(e))?
                .len();
            if !params.dry_run() {
                std::fs::remove_file(f.path())
                    .map_err(|e| crate::Error::internal_error().with_internal_data(e))?;
            }

            total_size += size;
        }

        Ok(ClearLogsResult::new(
            params.dry_run(),
            log_files.len(),
            total_size,
        ))
    }

    fn validate_user_is_admin(&self, user: &methods::User) -> Result<(), crate::Error> {
        if self.user_db.validate_user(user) {
            // username and password match
            if self
                .user_db
                .get_user(user.username())?
                .map(|user| *user.role())
                .unwrap_or(UserRole::User)
                < UserRole::Admin
            {
                Err(crate::Error::internal_error().with_data("you do not have permission"))
            } else {
                Ok(())
            }
        } else {
            Err(crate::Error::internal_error().with_data("invalid username or password"))
        }
    }
}
