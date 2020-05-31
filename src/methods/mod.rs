pub use prediction::PredictionController;
pub use server::ServerController;
use std::{fmt::Display, str::FromStr};
pub use user::{User, UserController};

mod prediction;
mod server;
mod user;

const SLEEP: &str = "sleep";
const ADD_PREDICTION: &str = "add_prediction";
const DELETE_PREDICTION: &str = "delete_prediction";
const SEARCH_PREDICTION: &str = "search_predictions";
const ADD_USER: &str = "add_user";
const CHANGE_PASSWORD: &str = "change_password";
const VALIDATE_USER: &str = "validate_user";
const SET_ROLE: &str = "set_role";
const CLEAR_LOGS: &str = "clear_logs";

pub enum Method {
    /// Sleep for a specified amount of time
    Sleep,
    /// Add a prediction to the database
    AddPrediction,
    /// Delete a prediction by its database id
    DeletePrediction,
    /// Search predictions
    SearchPredictions,
    /// Add a user
    AddUser,
    /// Change password for a user
    ChangePassword,
    /// Validate a username, password tuple
    ValidateUser,
    /// Set the role of a given user
    SetRole,
    /// Clear webserver logs
    ClearLogs,
}

impl FromStr for Method {
    type Err = (); // any failure means the method simply doesn't exist
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ADD_PREDICTION => Ok(Self::AddPrediction),
            DELETE_PREDICTION => Ok(Self::DeletePrediction),
            SEARCH_PREDICTION => Ok(Self::SearchPredictions),
            ADD_USER => Ok(Self::AddUser),
            CHANGE_PASSWORD => Ok(Self::ChangePassword),
            VALIDATE_USER => Ok(Self::ValidateUser),
            SET_ROLE => Ok(Self::SetRole),
            SLEEP => Ok(Self::Sleep),
            CLEAR_LOGS => Ok(Self::ClearLogs),
            _ => Err(()),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Sleep => SLEEP,
                Method::AddPrediction => ADD_PREDICTION,
                Method::DeletePrediction => DELETE_PREDICTION,
                Method::SearchPredictions => SEARCH_PREDICTION,
                Method::AddUser => ADD_USER,
                Method::ChangePassword => CHANGE_PASSWORD,
                Method::ValidateUser => VALIDATE_USER,
                Method::SetRole => SET_ROLE,
                Method::ClearLogs => CLEAR_LOGS,
            }
        )
    }
}
