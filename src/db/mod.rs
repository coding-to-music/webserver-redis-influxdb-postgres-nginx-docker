use ring::digest;
use rusqlite::{params, Connection};
use std::convert::From;
use std::marker::PhantomData;
use std::{fmt::Display, num::NonZeroU32, str::FromStr, time};

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

    pub fn get_connection(&self) -> Result<Connection, rusqlite::Error> {
        Connection::open(&self.path)
    }
}

impl Database<User> {
    pub fn username_exists(&self, username: &str) -> bool {
        if let Ok(Some(_user)) = self.get_user(username) {
            true
        } else {
            false
        }
    }

    pub fn add_user(&self, user: User) -> Result<usize, rusqlite::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO user (username, password, salt, created_s, role) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user.username, user.password, user.salt, user.created_s, user.role.to_string()],
        )?;

        Ok(changed_rows)
    }

    pub fn update_user_password(&self, user: User) -> Result<bool, rusqlite::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "UPDATE user SET password = ?1 WHERE username = ?2",
            params![user.password, user.username],
        )?;

        Ok(changed_rows == 1)
    }

    pub fn update_user_role(
        &self,
        username: &str,
        role: UserRole,
    ) -> Result<bool, rusqlite::Error> {
        let db = self.get_connection()?;
        let changed_rows = db.execute(
            "UPDATE user SET role = ?1 WHERE username = ?2",
            params![role.to_string(), username],
        )?;

        Ok(changed_rows == 1)
    }

    pub fn get_user(&self, username: &str) -> Result<Option<User>, rusqlite::Error> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "SELECT ROWID, username, password, salt, created_s, role FROM user WHERE username = ?1",
        )?;

        let mut user_rows: Vec<_> = stmt
            .query_map(params![username], |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password: row.get(2)?,
                    salt: row.get(3)?,
                    created_s: row.get(4)?,
                    role: UserRole::from_str(&row.get::<_, String>(5)?)
                        .unwrap_or_else(|_| UserRole::User),
                })
            })?
            .filter_map(|b| b.ok())
            .collect();

        if user_rows.is_empty() {
            Ok(None)
        } else if user_rows.len() > 1 {
            error!(r#"more than 1 user with username: "{}""#, username);
            Ok(None)
        } else {
            Ok(Some(user_rows.swap_remove(0)))
        }
    }

    pub fn validate_user(&self, user: &crate::methods::User) -> bool {
        trace!("validating user: {}", user.username());
        let user_row = if let Ok(Some(user)) = self.get_user(user.username()) {
            trace!(r#"user "{}" exists"#, user.username());
            user
        } else {
            trace!(r#"user "{}" does not exist"#, user.username());
            return false;
        };

        let salt_array = user_row.salt();

        let encrypted_password = Database::<User>::encrypt(user.password(), &salt_array);

        let valid = encrypted_password
            .iter()
            .zip(user_row.password().iter())
            .all(|(left, right)| left == right);

        trace!(
            r#"user "{}" is {}a valid user"#,
            user.username(),
            if valid { "" } else { "not " }
        );

        valid
    }

    pub fn encrypt(
        password: &str,
        salt: &[u8; digest::SHA512_OUTPUT_LEN],
    ) -> [u8; digest::SHA512_OUTPUT_LEN] {
        let timer = time::Instant::now();
        let mut hash = [0u8; digest::SHA512_OUTPUT_LEN];

        ring::pbkdf2::derive(
            ring::pbkdf2::PBKDF2_HMAC_SHA512,
            NonZeroU32::new(100_000).unwrap(),
            salt,
            password.as_bytes(),
            &mut hash,
        );

        trace!("encrypted a password in {:?}", timer.elapsed());

        hash
    }
}

impl Database<Prediction> {
    pub fn get_predictions_by_id(&self, id: i64) -> Result<Option<Prediction>, rusqlite::Error> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "SELECT ROWID, username, text, timestamp_s FROM prediction WHERE ROWID = ?1",
        )?;

        let mut prediction_rows: Vec<_> = stmt
            .query_map(params![id], |row| {
                let rowid = row.get(0)?;
                let username = row.get(1)?;
                let text = row.get(2)?;
                let timestamp_s = row.get(3)?;
                Ok(Prediction::new(rowid, username, text, timestamp_s))
            })?
            .filter_map(|r| r.ok())
            .collect();

        if prediction_rows.is_empty() {
            Ok(None)
        } else if prediction_rows.len() == 1 {
            Ok(Some(prediction_rows.swap_remove(0)))
        } else {
            error!("more than 1 prediction with id {}", id);
            Ok(None)
        }
    }

    pub fn insert_prediction(&self, prediction: Prediction) -> Result<bool, rusqlite::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO prediction (username, text, timestamp_s) VALUES (?1, ?2, ?3)",
            params![prediction.username, prediction.text, prediction.timestamp_s,],
        )?;

        Ok(changed_rows == 1)
    }

    pub fn delete_prediction(&self, id: i64) -> Result<bool, rusqlite::Error> {
        let db = self.get_connection()?;

        let changed_rows = db.execute("DELETE FROM prediction WHERE rowid = ?1", params![id])?;

        Ok(changed_rows == 1)
    }

    pub fn get_predictions_by_user(
        &self,
        username: &str,
    ) -> Result<Vec<Prediction>, rusqlite::Error> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "SELECT rowid, username, text, timestamp_s FROM prediction WHERE username = ?1",
        )?;

        let user_rows: Vec<_> = stmt
            .query_map(params![username], |row| {
                let rowid = row.get(0)?;
                let username = row.get(1)?;
                let text = row.get(2)?;
                let timestamp_s = row.get(3)?;
                Ok(Prediction::new(Some(rowid), username, text, timestamp_s))
            })?
            .filter_map(|b| b.ok())
            .collect();

        Ok(user_rows)
    }
}

pub struct Prediction {
    id: Option<i64>,
    username: String,
    text: String,
    timestamp_s: u32,
}

impl Prediction {
    pub fn new(id: Option<i64>, username: String, text: String, timestamp_s: u32) -> Self {
        Self {
            id,
            username,
            text,
            timestamp_s,
        }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn timestamp_s(&self) -> u32 {
        self.timestamp_s
    }
}

pub struct User {
    id: Option<i64>,
    username: String,
    password: Vec<u8>,
    salt: Vec<u8>,
    created_s: u32,
    role: UserRole,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserRole {
    User,
    Admin,
}

impl PartialOrd for UserRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        u16::from(self).partial_cmp(&u16::from(other))
    }
}

impl From<UserRole> for u16 {
    fn from(user_role: UserRole) -> Self {
        u16::from(&user_role)
    }
}

impl From<&UserRole> for u16 {
    fn from(user_role: &UserRole) -> Self {
        match user_role {
            UserRole::User => 100,
            UserRole::Admin => 200,
        }
    }
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UserRole::User => "user",
                UserRole::Admin => "admin",
            }
        )
    }
}

impl FromStr for UserRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "user" => UserRole::User,
            "admin" => UserRole::Admin,
            _ => return Err(()),
        })
    }
}

impl User {
    pub fn new(
        id: Option<i64>,
        username: String,
        password: Vec<u8>,
        salt: Vec<u8>,
        created_s: u32,
    ) -> Self {
        Self {
            id,
            username,
            password,
            salt,
            created_s,
            role: UserRole::User,
        }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
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

    pub fn created_s(&self) -> u32 {
        self.created_s
    }

    pub fn role(&self) -> &UserRole {
        &self.role
    }
}
