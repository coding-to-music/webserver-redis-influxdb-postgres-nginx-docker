use std::convert::TryInto;

use add::*;
use subtract::*;

pub struct MathController {}

impl MathController {
    pub fn new() -> Self {
        info!("Creating new MathController");
        Self {}
    }

    pub fn add<T: TryInto<AddParams, Error = AddParamsInvalid>>(
        &self,
        params: T,
    ) -> Result<AddResult, super::Error> {
        let params: AddParams = params.try_into()?;

        let sum = params.a() + params.b();

        Ok(AddResult::new(sum.into()))
    }

    pub fn subtract<T: TryInto<SubtractParams, Error = SubtractParamsInvalid>>(
        &self,
        params: T,
    ) -> Result<SubtractResult, super::Error> {
        let params: SubtractParams = params.try_into()?;

        let diff = params.a() - params.b();

        Ok(SubtractResult::new(diff.into()))
    }
}

mod add {
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct AddParams {
        a: i32,
        b: i32,
    }

    impl AddParams {
        pub fn a(&self) -> i32 {
            self.a
        }

        pub fn b(&self) -> i32 {
            self.b
        }
    }

    pub enum AddParamsInvalid {
        InvalidFormat,
    }

    impl TryFrom<serde_json::Value> for AddParams {
        type Error = AddParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params =
                serde_json::from_value(value).map_err(|_| AddParamsInvalid::InvalidFormat)?;

            Ok(params)
        }
    }

    #[derive(serde::Serialize)]
    pub struct AddResult {
        sum: i64,
    }

    impl AddResult {
        pub fn new(sum: i64) -> Self {
            Self { sum }
        }
    }

    impl From<AddParamsInvalid> for crate::methods::Error {
        fn from(_: AddParamsInvalid) -> Self {
            Self::invalid_params()
        }
    }
}

mod subtract {
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct SubtractParams {
        a: i32,
        b: i32,
    }

    impl SubtractParams {
        pub fn a(&self) -> i32 {
            self.a
        }

        pub fn b(&self) -> i32 {
            self.b
        }
    }

    pub enum SubtractParamsInvalid {
        InvalidFormat,
    }

    impl TryFrom<serde_json::Value> for SubtractParams {
        type Error = SubtractParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params =
                serde_json::from_value(value).map_err(|_| SubtractParamsInvalid::InvalidFormat)?;

            Ok(params)
        }
    }

    #[derive(serde::Serialize)]
    pub struct SubtractResult {
        difference: i64,
    }

    impl SubtractResult {
        pub fn new(difference: i64) -> Self {
            Self { difference }
        }
    }

    impl From<SubtractParamsInvalid> for crate::methods::Error {
        fn from(_: SubtractParamsInvalid) -> Self {
            Self::invalid_params()
        }
    }
}
