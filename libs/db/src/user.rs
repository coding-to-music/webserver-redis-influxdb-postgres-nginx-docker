use crate::{Database, DatabaseError};
use rusqlite::{params, Row};
use std::{convert::TryFrom, fmt::Display, str::FromStr};

pub const PASSWORD_BYTE_LEN: usize = 64;
pub const SALT_BYTE_LEN: usize = 64;
const USER: &str = "user";
const ADMIN: &str = "admin";

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

    fn password(&self) -> &[u8; PASSWORD_BYTE_LEN] {
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
