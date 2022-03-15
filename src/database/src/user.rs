use sqlx::{FromRow, types::time::OffsetDateTime};

use crate::{Database, DatabaseResult, InsertionResult};

pub type UserDatabase = Database<User>;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
#[non_exhaustive]
pub struct User {
    pub id: String,
    pub username: String,
    pub created: OffsetDateTime,
}

impl UserDatabase {
    pub async fn insert_user(
        &self,
        id: &str,
        username: &str,
        password: &str,
    ) -> DatabaseResult<InsertionResult> {
        let mut db = self.get_connection().await?;

        let query_result = sqlx::query(
            "INSERT INTO user (id, username, password) VALUES ($1, $2, crypt($3, gen_salt('bf')))",
        )
        .bind(id)
        .bind(username)
        .bind(password)
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
            sqlx::query_as::<_, User>(r#"SELECT id, username, created FROM "user" WHERE username = $1 AND password = crypt($2, password)"#)
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

    pub async fn get_roles_for_user(&self, id: &str) -> DatabaseResult<Vec<String>> {
        let mut db = self.get_connection().await?;

        #[derive(FromRow)]
        struct RoleName {
            role_name: String,
        };

        let mut query_result = sqlx::query_as::<_, RoleName>(
            r#"
            SELECT R.NAME AS ROLE_NAME
            FROM "user" U
            JOIN USER_ROLE UR ON UR.USER_ID = U.ID
            JOIN "role" R ON UR.ROLE_ID = R.ID
            WHERE U.id = $1"#,
        )
        .bind(id)
        .fetch_all(&mut db)
        .await?;

        Ok(query_result.into_iter().map(|rn| rn.role_name).collect())
    }
}
