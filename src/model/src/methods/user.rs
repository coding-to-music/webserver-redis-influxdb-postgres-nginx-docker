pub mod add_user;
pub mod get_token;
pub mod get_user;

#[derive(serde::Serialize, Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct User {
    pub id: String,
    pub username: String,
}

impl User {
    pub fn new(id: String, username: String) -> Self {
        Self { id, username }
    }
}
