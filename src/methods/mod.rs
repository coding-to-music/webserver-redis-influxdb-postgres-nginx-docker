pub use prediction::PredictionController;
pub use sleep::SleepController;
use std::str::FromStr;
pub use user::{User, UserController};

mod prediction;
mod sleep;
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
            "sleep" => Ok(Self::Sleep),
            _ => Err(()),
        }
    }
}
