use super::Database;
use rand::SystemRandom;
use ring::{
    digest,
    rand::{self, SecureRandom},
};
use rusqlite::{params, Connection};
use std::convert::TryInto;
use std::num::NonZeroU32;

pub struct UserController {
    db_path: String,
}

impl Database for UserController {
    fn get_connection(&self) -> Result<Connection, super::Error> {
        let db = Connection::open(&self.db_path).map_err(|e| {
            error!("Failed to connect to database: {:?}", e);
            super::Error::internal_error()
        })?;

        info!("Connected to db");
        Ok(db)
    }
}

impl UserController {
    pub fn new(db_path: String) -> Self {
        info!("Creating new UserController");
        Self { db_path }
    }

    fn add_user(&self, user: UserRow) -> Result<usize, crate::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO user (username, password, salt) VALUES (?1, ?2, ?3)",
            params![user.username, user.password, user.salt],
        )?;

        Ok(changed_rows)
    }

    fn get_user(&self, username: &str) -> Result<Option<UserRow>, crate::Error> {
        let db = self.get_connection()?;

        let mut stmt = db
            .prepare("SELECT username, password, salt FROM user WHERE username = ?1")
            .map_err(|e| {
                error!("{:?}", e);
                super::Error::internal_error()
            })?;

        let mut user_rows: Vec<_> = stmt
            .query_map(params![username], |row| {
                Ok(UserRow::new(row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|b| b.ok())
            .collect();

        if user_rows.is_empty() {
            Ok(None)
        } else if user_rows.len() > 1 {
            error!(r#"more than 1 user with username: "{}""#, username);
            Err(crate::Error::internal_error())
        } else {
            Ok(Some(user_rows.swap_remove(0)))
        }
    }

    pub async fn add<T: TryInto<add::AddUserParams, Error = add::AddUserParamsInvalid>>(
        &self,
        request: T,
    ) -> Result<add::AddUserResult, crate::Error> {
        let params: add::AddUserParams = request.try_into()?;

        if let Ok(Some(_)) = self.get_user(params.user().username()) {
            return Err(crate::Error::internal_error()
                .with_data("a user with that username already exists"));
        }

        let rng = SystemRandom::new();
        let mut salt = [0u8; digest::SHA512_OUTPUT_LEN];

        rng.fill(&mut salt).map_err(|e| {
            error!("{}", e);
            crate::Error::internal_error()
        })?;

        let hashed_password = self.encrypt(params.user().password(), salt);

        let user_row = UserRow::new(
            params.user().username().to_owned(),
            hashed_password.to_vec(),
            salt.to_vec(),
        );

        let rows = self.add_user(user_row)?;

        Ok(add::AddUserResult::new(rows == 1))
    }

    fn encrypt(
        &self,
        password: &str,
        salt: [u8; digest::SHA512_OUTPUT_LEN],
    ) -> [u8; digest::SHA512_OUTPUT_LEN] {
        let mut hash = [0u8; digest::SHA512_OUTPUT_LEN];

        ring::pbkdf2::derive(
            ring::pbkdf2::PBKDF2_HMAC_SHA512,
            NonZeroU32::new(100_000).unwrap(),
            &salt,
            password.as_bytes(),
            &mut hash,
        );

        hash
    }
}

pub struct UserRow {
    username: String,
    password: Vec<u8>,
    salt: Vec<u8>,
}

impl UserRow {
    pub fn new(username: String, password: Vec<u8>, salt: Vec<u8>) -> Self {
        Self {
            username,
            password,
            salt,
        }
    }
}

mod add {
    use super::*;
    use std::convert::TryFrom;

    #[derive(serde::Deserialize)]
    pub struct AddUserParams {
        user: User,
    }

    impl AddUserParams {
        pub fn user(&self) -> &User {
            &self.user
        }
    }

    #[derive(serde::Deserialize)]
    pub struct User {
        username: String,
        password: String,
    }

    impl User {
        pub fn username(&self) -> &str {
            &self.username
        }

        pub fn password(&self) -> &str {
            &self.password
        }
    }

    pub enum AddUserParamsInvalid {
        InvalidFormat,
        PasswordTooShort,
    }

    impl TryFrom<serde_json::Value> for AddUserParams {
        type Error = AddUserParamsInvalid;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            let params: AddUserParams =
                serde_json::from_value(value).map_err(|_| AddUserParamsInvalid::InvalidFormat)?;

            if params.user.password.len() < 10 {
                Err(AddUserParamsInvalid::PasswordTooShort)
            } else {
                Ok(params)
            }
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for AddUserParams {
        type Error = AddUserParamsInvalid;
        fn try_from(value: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            value.params.try_into()
        }
    }

    impl From<AddUserParamsInvalid> for crate::Error {
        fn from(_: AddUserParamsInvalid) -> Self {
            Self::invalid_params()
        }
    }

    #[derive(serde::Serialize)]
    pub struct AddUserResult {
        success: bool,
    }

    impl AddUserResult {
        pub fn new(success: bool) -> Self {
            Self { success }
        }
    }
}
