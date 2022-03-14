use crate::{Database, DatabaseResult, InsertionResult};

pub type UserDatabase = Database<User>;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
#[non_exhaustive]
pub struct User {
    id: String,
    username: String,
    created_s: i64,
}

impl UserDatabase {
    pub async fn insert_user(
        &self,
        id: &str,
        username: &str,
        password: &str,
        created_s: i64,
    ) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query(
            "INSERT INTO user (id, username, password, created_s) VALUES ($1, $2, crypt($3, gen_salt('bf')), $4)",
        )
        .bind(id)
        .bind(username)
        .bind(password)
        .bind(created_s)
        .execute(&mut db)
        .await?;

        Ok(InsertionResult::from_changed_rows(
            query_result.rows_affected(),
        ))
    }

    pub async fn get_user_by_id(&self, id: &str) -> DatabaseResult<Option<User>> {
        let mut db = self.get_connection().await?;

        let mut query_result =
            sqlx::query_as::<_, User>("SELECT id, username, created_s FROM user WHERE id = $1")
                .bind(id)
                .fetch_all(&mut db)
                .await?;

        if query_result.is_empty() {
            Ok(None)
        } else if query_result.len() > 1 {
            error!(r#"more than 1 user with id: "{}""#, id);
            Ok(None)
        } else {
            Ok(Some(query_result.remove(0)))
        }
    }

    pub async fn validate_user(
        &self,
        username: &str,
        password: &str,
    ) -> DatabaseResult<Option<User>> {
        let mut db = self.get_connection().await?;

        let mut query_result =
            sqlx::query_as::<_, User>("SELECT id, username, created_s FROM user WHERE username = $1 AND password = crypt($2, password)")
                .bind(username)
                .bind(password)
                .fetch_all(&mut db)
                .await?;

        if query_result.is_empty() {
            Ok(None)
        } else if query_result.len() > 1 {
            error!(r#"more than 1 user with username: "{}""#, username);
            Ok(None)
        } else {
            Ok(Some(query_result.remove(0)))
        }
    }
}
