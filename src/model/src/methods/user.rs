pub mod add_user;
pub mod get_token;

#[derive(serde::Serialize, Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct User {
    pub id: String,
    pub username: String,
}
