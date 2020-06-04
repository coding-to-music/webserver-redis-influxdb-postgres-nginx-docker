use crate::AppError;
use db::{Database, User, UserRole};
use std::{convert::TryFrom, path::PathBuf, sync::Arc, time};
use time::Duration;
use tokio::sync::Mutex;
use webserver_contracts::{server::*, Error as JsonRpcError};

pub struct ServerController {
    user_db: Arc<Database<User>>,
    log_directory: PathBuf,
    served_requests: Mutex<u32>,
}

impl ServerController {
    pub fn new(user_db: Arc<Database<User>>, log_directory: PathBuf) -> Self {
        Self {
            user_db,
            log_directory,
            served_requests: Mutex::new(0),
        }
    }

    pub async fn sleep(&self, request: crate::JsonRpcRequest) -> Result<SleepResult, AppError> {
        let params = SleepParams::try_from(request)?;
        if !self
            .user_db
            .get_user(params.user().username())?
            .map(|u| u.is_authorized(UserRole::Admin))
            .unwrap_or(false)
        {
            return Err(AppError::from(JsonRpcError::not_permitted()));
        }

        let now = std::time::Instant::now();
        tokio::time::delay_for(Duration::from_secs_f32(params.seconds())).await;
        let elapsed = now.elapsed();

        self.increment_served_requests().await;

        Ok(SleepResult::new(elapsed.as_secs_f32()))
    }

    pub async fn clear_logs(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<ClearLogsResult, AppError> {
        let params = ClearLogsParams::try_from(request)?;

        if !self
            .user_db
            .get_user(params.user().username())?
            .map(|u| u.is_authorized(UserRole::Admin))
            .unwrap_or(false)
        {
            return Err(AppError::from(JsonRpcError::not_permitted()));
        }

        let paths =
            std::fs::read_dir(&self.log_directory).map_err(|_e| JsonRpcError::internal_error())?;

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
                .map_err(|_e| JsonRpcError::internal_error())?
                .len();
            if !params.dry_run() {
                std::fs::remove_file(f.path()).map_err(|_e| JsonRpcError::internal_error())?;
            }

            total_size += size;
        }

        self.increment_served_requests().await;

        Ok(ClearLogsResult::new(
            params.dry_run(),
            log_files.len(),
            total_size,
        ))
    }

    async fn increment_served_requests(&self) {
        let mut served = self.served_requests.lock().await;
        trace!(
            "incrementing served requests from {} to {}...",
            served,
            *served + 1
        );
        *served += 1;
        trace!("incremented served requests")
    }
}

impl From<SleepParamsInvalid> for AppError {
    fn from(error: SleepParamsInvalid) -> Self {
        match error {
            SleepParamsInvalid::InvalidFormat(e) => AppError::from(JsonRpcError::invalid_format(e)),
            SleepParamsInvalid::SecondsTooLow => {
                AppError::from(JsonRpcError::invalid_params().with_data("'seconds' too low"))
            }
            SleepParamsInvalid::SecondsTooHigh => {
                AppError::from(JsonRpcError::invalid_params().with_data("'seconds' too high"))
            }
        }
    }
}

impl From<ClearLogsParamsInvalid> for AppError {
    fn from(error: ClearLogsParamsInvalid) -> Self {
        match error {
            ClearLogsParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}
