use crate::AppError;
use std::{convert::TryFrom, path::PathBuf, sync::Arc, time};
use time::Duration;
use tokio::sync::Mutex;
use webserver_contracts::{server::*, user, Error as JsonRpcError};
use webserver_database as db;

pub struct ServerController {
    user_db: Arc<db::Database<db::User>>,
    log_directory: PathBuf,
    served_requests: Mutex<u32>,
}

impl ServerController {
    pub fn new(user_db: Arc<db::Database<db::User>>, log_directory: PathBuf) -> Self {
        Self {
            user_db,
            log_directory,
            served_requests: Mutex::new(0),
        }
    }

    pub async fn sleep(&self, request: crate::JsonRpcRequest) -> Result<SleepResult, AppError> {
        let params = SleepParams::try_from(request)?;
        self.authorize_admin(&params.user)?;

        let elapsed = if params.sync {
            let now = std::time::Instant::now();
            std::thread::sleep(Duration::from_secs_f32(params.seconds));
            now.elapsed()
        } else {
            let now = std::time::Instant::now();
            tokio::time::delay_for(Duration::from_secs_f32(params.seconds)).await;
            now.elapsed()
        };

        self.increment_served_requests().await;

        Ok(SleepResult::new(elapsed.as_secs_f32()))
    }

    pub async fn clear_logs(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<ClearLogsResult, AppError> {
        let params = ClearLogsParams::try_from(request)?;

        self.authorize_admin(&params.user)?;

        let paths =
            std::fs::read_dir(&self.log_directory).map_err(|_e| JsonRpcError::internal_error())?;

        let log_files: Vec<_> = paths
            .filter_map(|p| p.ok())
            .filter(|p| p.metadata().map(|m| m.is_file()).unwrap_or(false))
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
            if !params.dry_run {
                std::fs::remove_file(f.path()).map_err(|_e| JsonRpcError::internal_error())?;
            }

            total_size += size;
        }

        self.increment_served_requests().await;

        Ok(ClearLogsResult::new(
            params.dry_run,
            log_files.len(),
            total_size,
        ))
    }

    pub async fn prepare_tests(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<PrepareTestsResult, AppError> {
        let params = PrepareTestsParams::try_from(request)?;

        self.authorize_admin(&params.user)?;

        Ok(PrepareTestsResult::new(false))
    }

    pub async fn get_all_usernames(
        &self,
        request: crate::JsonRpcRequest,
    ) -> Result<GetAllUsernamesResult, AppError> {
        let params = GetAllUsernamesParams::try_from(request)?;
        self.authorize_admin(&params.user)?;

        let result = self.user_db.get_all_usernames()?;

        Ok(GetAllUsernamesResult::new(result))
    }

    fn authorize_admin(&self, user: &user::User) -> Result<(), AppError> {
        debug!("validating user {:?}", user);
        match self.user_db.get_user_by_username(&user.username)?.map(|u| {
            let encrypted_password = crate::encrypt(user.password.as_bytes(), &u.salt);
            (
                u.validate_password(&encrypted_password),
                u.is_authorized(db::UserRole::Admin),
            )
        }) {
            // tuple is (password is correct, user is admin)
            Some((true, true)) => {
                debug!("password is valid and user is admin");
                Ok(())
            }
            Some((true, false)) => {
                debug!("password is valid but user is not admin");
                Err(AppError::from(JsonRpcError::not_permitted()))
            }
            Some((false, _)) => {
                debug!("password is invalid");
                Err(AppError::from(JsonRpcError::invalid_username_or_password()))
            }
            None => {
                debug!("user does not exist");
                Err(AppError::from(
                    JsonRpcError::internal_error().with_data("user does not exist"),
                ))
            }
        }
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

impl From<PrepareTestsParamsInvalid> for AppError {
    fn from(error: PrepareTestsParamsInvalid) -> Self {
        match error {
            PrepareTestsParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}

impl From<GetAllUsernamesParamsInvalid> for AppError {
    fn from(error: GetAllUsernamesParamsInvalid) -> Self {
        match error {
            GetAllUsernamesParamsInvalid::InvalidFormat(e) => {
                AppError::from(JsonRpcError::invalid_format(e))
            }
        }
    }
}
