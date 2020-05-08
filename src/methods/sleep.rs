use std::convert::TryInto;

pub struct SleepController;

impl SleepController {
    pub fn new() -> Self {
        info!("Creating new SleepController");
        Self
    }

    pub async fn sleep<T: TryInto<sleep::SleepParams, Error = sleep::SleepParamsInvalid>>(
        &self,
        request: T,
    ) -> Result<sleep::SleepResult, crate::Error> {
        let params: sleep::SleepParams = request.try_into()?;

        let now = std::time::Instant::now();
        tokio::time::delay_for(std::time::Duration::from_secs_f32(params.seconds())).await;
        let elapsed = now.elapsed();

        Ok(sleep::SleepResult::new(elapsed.as_secs_f32()))
    }
}

mod sleep {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct SleepParams {
        seconds: f32,
    }

    impl SleepParams {
        pub fn seconds(&self) -> f32 {
            self.seconds
        }
    }

    impl TryFrom<serde_json::Value> for SleepParams {
        type Error = SleepParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params: SleepParams =
                serde_json::from_value(value).map_err(|_| SleepParamsInvalid::InvalidFormat)?;

            if params.seconds < 0.01 {
                Err(SleepParamsInvalid::SecondsTooLow)
            } else if params.seconds > 10.0 {
                Err(SleepParamsInvalid::SecondsTooHigh)
            } else {
                Ok(params)
            }
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for SleepParams {
        type Error = SleepParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            value.params.try_into()
        }
    }

    impl From<SleepParamsInvalid> for crate::methods::Error {
        fn from(error: SleepParamsInvalid) -> Self {
            match error {
                SleepParamsInvalid::InvalidFormat => Self::invalid_params(),
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
        InvalidFormat,
        SecondsTooLow,
        SecondsTooHigh,
    }

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct SleepResult {
        seconds: f32,
    }

    impl SleepResult {
        pub fn new(seconds: f32) -> Self {
            Self { seconds }
        }
    }
}
