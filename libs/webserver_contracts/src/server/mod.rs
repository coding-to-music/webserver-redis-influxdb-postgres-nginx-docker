pub use clear_logs::{ClearLogsParams, ClearLogsParamsInvalid, ClearLogsResult};
pub use sleep::{SleepParams, SleepParamsInvalid, SleepResult};

mod sleep {
    use crate::user::User;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    struct SleepParamsBuilder {
        seconds: f32,
        user: User,
    }

    impl SleepParamsBuilder {
        pub fn build(self) -> Result<SleepParams, SleepParamsInvalid> {
            if self.seconds < 0.01 {
                Err(SleepParamsInvalid::SecondsTooLow)
            } else if self.seconds > 10.0 {
                Err(SleepParamsInvalid::SecondsTooHigh)
            } else {
                Ok(SleepParams {
                    seconds: self.seconds,
                    user: self.user,
                })
            }
        }
    }

    pub struct SleepParams {
        user: User,
        seconds: f32,
    }

    impl SleepParams {
        pub fn user(&self) -> &User {
            &self.user
        }

        pub fn seconds(&self) -> f32 {
            self.seconds
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for SleepParams {
        type Error = SleepParamsInvalid;
        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: SleepParamsBuilder = serde_json::from_value(request.params)
                .map_err(SleepParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    impl From<SleepParamsInvalid> for crate::Error {
        fn from(error: SleepParamsInvalid) -> Self {
            match error {
                SleepParamsInvalid::InvalidFormat(e) => Self::invalid_format(e),
                SleepParamsInvalid::SecondsTooLow => {
                    Self::invalid_params().with_data("can't sleep for less than 0.01 seconds")
                }
                SleepParamsInvalid::SecondsTooHigh => {
                    Self::invalid_params().with_data("can't sleep for more than 10.0 seconds")
                }
            }
        }
    }

    pub enum SleepParamsInvalid {
        InvalidFormat(serde_json::Error),
        SecondsTooLow,
        SecondsTooHigh,
    }

    #[derive(serde::Serialize)]
    pub struct SleepResult {
        seconds: f32,
    }

    impl SleepResult {
        pub fn new(seconds: f32) -> Self {
            Self { seconds }
        }
    }
}

mod clear_logs {
    use crate::user::User;
    use std::convert::TryFrom;

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct ClearLogsParams {
        user: User,
        dry_run: bool,
    }

    impl ClearLogsParams {
        pub fn user(&self) -> &User {
            &self.user
        }

        pub fn dry_run(&self) -> bool {
            self.dry_run
        }
    }

    #[derive(serde::Deserialize)]
    struct ClearLogsParamsBuilder {
        user: User,
        dry_run: bool,
    }

    impl ClearLogsParamsBuilder {
        fn build(self) -> Result<ClearLogsParams, ClearLogsParamsInvalid> {
            Ok(ClearLogsParams {
                user: self.user,
                dry_run: self.dry_run,
            })
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for ClearLogsParams {
        type Error = ClearLogsParamsInvalid;
        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: ClearLogsParamsBuilder = serde_json::from_value(request.params)
                .map_err(ClearLogsParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    impl From<ClearLogsParamsInvalid> for crate::Error {
        fn from(error: ClearLogsParamsInvalid) -> Self {
            match error {
                ClearLogsParamsInvalid::InvalidFormat(e) => Self::invalid_format(e),
            }
        }
    }

    pub enum ClearLogsParamsInvalid {
        InvalidFormat(serde_json::Error),
    }

    #[derive(serde::Serialize)]
    pub struct ClearLogsResult {
        dry_run: bool,
        files: usize,
        bytes: u64,
    }

    impl ClearLogsResult {
        pub fn new(dry_run: bool, files: usize, bytes: u64) -> Self {
            Self {
                dry_run,
                files,
                bytes,
            }
        }
    }
}
