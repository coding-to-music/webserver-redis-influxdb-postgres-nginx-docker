use ring::digest;
use rusqlite::{params, Connection};
use std::marker::PhantomData;
use std::num::NonZeroU32;

pub struct Database<T> {
    path: String,
    _phantom: PhantomData<T>,
}

impl<T> Database<T> {
    pub fn new(path: String) -> Self {
        Self {
            path,
            _phantom: PhantomData,
        }
    }

    pub fn get_connection(&self) -> Result<Connection, crate::Error> {
        let db = Connection::open(&self.path).map_err(|e| {
            error!("Failed to connect to database: {:?}", e);
            super::Error::internal_error()
        })?;

        info!("Connected to db");
        Ok(db)
    }
}

impl Database<User> {
    pub fn add_user(&self, user: User) -> Result<usize, crate::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO user (username, password, salt) VALUES (?1, ?2, ?3)",
            params![user.username, user.password, user.salt],
        )?;

        Ok(changed_rows)
    }

    pub fn get_user(&self, username: &str) -> Result<Option<User>, crate::Error> {
        let db = self.get_connection()?;

        let mut stmt = db
            .prepare("SELECT username, password, salt FROM user WHERE username = ?1")
            .map_err(|e| {
                error!("{:?}", e);
                super::Error::internal_error()
            })?;

        let mut user_rows: Vec<_> = stmt
            .query_map(params![username], |row| {
                Ok(User::new(row.get(0)?, row.get(1)?, row.get(2)?))
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

    pub fn validate_user(&self, user: &crate::methods::User) -> bool {
        let user_row = if let Ok(Some(user)) = self.get_user(user.username()) {
            user
        } else {
            return false;
        };

        let salt_array = user_row.salt();

        let encrypted_password = Database::<User>::encrypt(user.password(), &salt_array);

        encrypted_password
            .iter()
            .zip(user_row.password().iter())
            .all(|(left, right)| left == right)
    }

    pub fn encrypt(
        password: &str,
        salt: &[u8; digest::SHA512_OUTPUT_LEN],
    ) -> [u8; digest::SHA512_OUTPUT_LEN] {
        let mut hash = [0u8; digest::SHA512_OUTPUT_LEN];

        ring::pbkdf2::derive(
            ring::pbkdf2::PBKDF2_HMAC_SHA512,
            NonZeroU32::new(100_000).unwrap(),
            salt,
            password.as_bytes(),
            &mut hash,
        );

        hash
    }
}

impl Database<Prediction> {
    pub fn insert_prediction(&self, prediction: Prediction) -> Result<bool, super::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO prediction (username, text, timestamp_s) VALUES (?1, ?2, ?3)",
            params![prediction.username, prediction.text, prediction.timestamp_s,],
        )?;

        Ok(changed_rows == 1)
    }
}

pub struct Prediction {
    username: String,
    text: String,
    timestamp_s: u32,
}

impl Prediction {
    pub fn new(username: String, text: String, timestamp_s: u32) -> Self {
        Self {
            username,
            text,
            timestamp_s,
        }
    }
}

pub struct User {
    username: String,
    password: Vec<u8>,
    salt: Vec<u8>,
}

impl User {
    pub fn new(username: String, password: Vec<u8>, salt: Vec<u8>) -> Self {
        Self {
            username,
            password,
            salt,
        }
    }

    pub fn password(&self) -> [u8; digest::SHA512_OUTPUT_LEN] {
        assert_eq!(self.password.len(), digest::SHA512_OUTPUT_LEN);

        let mut bytes = [0u8; digest::SHA512_OUTPUT_LEN];
        for (idx, byte) in self.password.iter().enumerate() {
            bytes[idx] = *byte;
        }

        bytes
    }

    pub fn salt(&self) -> [u8; digest::SHA512_OUTPUT_LEN] {
        assert_eq!(self.salt.len(), digest::SHA512_OUTPUT_LEN);

        let mut bytes = [0u8; digest::SHA512_OUTPUT_LEN];
        for (idx, byte) in self.salt.iter().enumerate() {
            bytes[idx] = *byte;
        }

        bytes
    }
}
