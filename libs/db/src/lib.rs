use rusqlite::{params, Connection, Row};
use std::convert::{From, TryFrom};
use std::marker::PhantomData;
use std::{fmt::Display, str::FromStr, time};

#[macro_use]
extern crate log;

pub struct Database<T> {
    path: String,
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub enum DatabaseError {
    RusqliteError(rusqlite::Error),
    NotAuthorized,
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(rusqlite_error: rusqlite::Error) -> Self {
        DatabaseError::RusqliteError(rusqlite_error)
    }
}

const USER: &str = "user";
const ADMIN: &str = "admin";
pub const PASSWORD_BYTE_LEN: usize = 64;
pub const SALT_BYTE_LEN: usize = 64;

impl<T> Database<T> {
    pub fn new(path: String) -> Self {
        Self {
            path,
            _phantom: PhantomData,
        }
    }

    fn get_connection(&self) -> Result<Connection, DatabaseError> {
        trace!("connecting to database at '{}'", self.path);
        let timer = time::Instant::now();
        let conn = Connection::open(&self.path).map_err(|e| DatabaseError::from(e));
        if conn.is_ok() {
            trace!(
                "successfully connected to database at '{}' in {:?}",
                self.path,
                timer.elapsed()
            );
        }
        conn
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

    pub fn add_user(&self, user: User) -> Result<usize, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO user (username, password, salt, created_s, role) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user.username, user.password.to_vec(), user.salt.to_vec(), user.created_s, user.role.to_string()],
        )?;

        Ok(changed_rows)
    }

    pub fn update_user_password(&self, user: User) -> Result<bool, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "UPDATE user SET password = ?1 WHERE username = ?2",
            params![user.password.to_vec(), user.username],
        )?;

        Ok(changed_rows == 1)
    }

    pub fn update_user_role(&self, username: &str, role: UserRole) -> Result<bool, DatabaseError> {
        let db = self.get_connection()?;
        let changed_rows = db.execute(
            "UPDATE user SET role = ?1 WHERE username = ?2",
            params![role.to_string(), username],
        )?;

        Ok(changed_rows == 1)
    }

    pub fn get_user(&self, username: &str) -> Result<Option<User>, DatabaseError> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "SELECT ROWID, username, password, salt, created_s, role FROM user WHERE username = ?1",
        )?;

        let mut user_rows: Vec<_> = stmt
            .query_map(params![username], |row| Ok(User::try_from(row)?))?
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

    pub fn get_all_usernames(&self) -> Result<Vec<String>, DatabaseError> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare("SELECT username FROM user")?;

        let user_rows: Vec<String> = stmt
            .query_map(params![], |row| Ok(row.get(0)?))?
            .filter_map(|b| b.ok())
            .collect();
            
        Ok(user_rows)
    }

    pub fn delete_user(&self, username: &str) -> Result<bool, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute("DELETE FROM user WHERE username = ?1", params![username])?;

        Ok(changed_rows == 1)
    }
}

impl Database<Prediction> {
    pub fn delete_predictions_by_username(&self, username: &str) -> Result<usize, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "DELETE FROM prediction WHERE username = ?1",
            params![username],
        )?;

        Ok(changed_rows)
    }

    pub fn get_predictions_by_id(&self, id: i64) -> Result<Option<Prediction>, DatabaseError> {
        let db = self.get_connection()?;

        let mut stmt = db.prepare(
            "SELECT ROWID, username, text, timestamp_s FROM prediction WHERE ROWID = ?1",
        )?;

        let mut prediction_rows: Vec<_> = stmt
            .query_map(params![id], |row| Ok(Prediction::try_from(row)?))?
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

    pub fn insert_prediction(&self, prediction: Prediction) -> Result<bool, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute(
            "INSERT INTO prediction (username, text, timestamp_s) VALUES (?1, ?2, ?3)",
            params![prediction.username, prediction.text, prediction.timestamp_s,],
        )?;

        Ok(changed_rows == 1)
    }

    pub fn delete_prediction(&self, id: i64) -> Result<bool, DatabaseError> {
        let db = self.get_connection()?;

        let changed_rows = db.execute("DELETE FROM prediction WHERE rowid = ?1", params![id])?;

        Ok(changed_rows == 1)
    }

    pub fn get_predictions_by_user(
        &self,
        username: &str,
    ) -> Result<Vec<Prediction>, DatabaseError> {
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

impl<'a> TryFrom<&Row<'a>> for Prediction {
    type Error = rusqlite::Error;
    fn try_from(row: &Row<'a>) -> Result<Self, Self::Error> {
        let rowid = row.get(0)?;
        let username = row.get(1)?;
        let text = row.get(2)?;
        let timestamp_s = row.get(3)?;
        Ok(Prediction::new(rowid, username, text, timestamp_s))
    }
}

pub struct User {
    id: Option<i64>,
    username: String,
    password: [u8; PASSWORD_BYTE_LEN],
    salt: [u8; SALT_BYTE_LEN],
    created_s: u32,
    role: UserRole,
}

impl<'a> TryFrom<&Row<'a>> for User {
    type Error = rusqlite::Error;
    fn try_from(row: &Row<'a>) -> Result<Self, Self::Error> {
        let password_vec: Vec<u8> = row.get(2)?;
        let salt_vec: Vec<u8> = row.get(3)?;

        assert_eq!(password_vec.len(), PASSWORD_BYTE_LEN);
        assert_eq!(salt_vec.len(), SALT_BYTE_LEN);

        let mut password_arr = [0_u8; PASSWORD_BYTE_LEN];
        for (idx, byte) in password_vec.into_iter().enumerate() {
            password_arr[idx] = byte;
        }

        let mut salt_arr = [0_u8; SALT_BYTE_LEN];
        for (idx, byte) in salt_vec.into_iter().enumerate() {
            salt_arr[idx] = byte;
        }

        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: password_arr,
            salt: salt_arr,
            created_s: row.get(4)?,
            role: UserRole::from_str(&row.get::<_, String>(5)?).unwrap_or_else(|_| UserRole::User),
        })
    }
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
                UserRole::User => USER,
                UserRole::Admin => ADMIN,
            }
        )
    }
}

impl FromStr for UserRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            USER => UserRole::User,
            ADMIN => UserRole::Admin,
            _ => return Err(()),
        })
    }
}

impl User {
    pub fn new(
        id: Option<i64>,
        username: String,
        password: [u8; PASSWORD_BYTE_LEN],
        salt: [u8; SALT_BYTE_LEN],
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

    pub fn password(&self) -> &[u8; PASSWORD_BYTE_LEN] {
        &self.password
    }

    pub fn salt(&self) -> &[u8; SALT_BYTE_LEN] {
        &self.salt
    }

    pub fn created_s(&self) -> u32 {
        self.created_s
    }

    pub fn role(&self) -> &UserRole {
        &self.role
    }

    pub fn is_authorized(&self, level: UserRole) -> bool {
        self.role >= level
    }

    pub fn validate_password(&self, compare: &[u8; PASSWORD_BYTE_LEN]) -> bool {
        compare
            .iter()
            .zip(self.password().iter())
            .all(|(left, right)| left == right)
    }
}
