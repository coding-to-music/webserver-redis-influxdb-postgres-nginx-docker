pub use prediction::PredictionController;
pub use server::ServerController;
use std::str::FromStr;
pub use user::{User, UserController};

mod prediction;
mod server;
mod user;

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
            "add_prediction" => Ok(Self::AddPrediction),
            "delete_prediction" => Ok(Self::DeletePrediction),
            "search_predictions" => Ok(Self::SearchPredictions),
            "add_user" => Ok(Self::AddUser),
            "change_password" => Ok(Self::ChangePassword),
            "validate_user" => Ok(Self::ValidateUser),
            "set_role" => Ok(Self::SetRole),
            "sleep" => Ok(Self::Sleep),
            "clear_logs" => Ok(Self::ClearLogs),
            _ => Err(()),
        }
    }
}
