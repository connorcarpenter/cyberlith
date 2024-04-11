use serde::{Deserialize, Serialize};
use std::fmt::Display;

use db::{DbRowKey, DbRowValue, DbTableKey};

// users table key
pub struct Users;

impl DbTableKey for Users {
    type Key = UserId;
    type Value = User;

    fn repo_name() -> &'static str {
        "cyberlith_users"
    }
}

// user id
#[derive(Serialize, Deserialize, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct UserId {
    id: u64,
}

impl DbRowKey for UserId {}

impl UserId {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        Self { id }
    }
}

impl Into<u64> for UserId {
    fn into(self) -> u64 {
        self.id
    }
}

// user role
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum UserRole {
    Admin,
    Staff,
    Paid,
    Free,
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "Admin"),
            UserRole::Staff => write!(f, "Staff"),
            UserRole::Paid => write!(f, "Paid"),
            UserRole::Free => write!(f, "Free"),
        }
    }
}

// user
#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    id: Option<u64>,
    name: String,
    email: String,
    password: String,
    create_date: chrono::DateTime<chrono::Utc>,
    role: UserRole,
}

impl User {}

impl DbRowValue for User {
    type Key = UserId;

    fn get_key(&self) -> <Self as DbRowValue>::Key {
        <Self as DbRowValue>::Key::from(self.id.unwrap())
    }

    fn set_key(&mut self, key: <Self as DbRowValue>::Key) {
        self.id = Some(key.into());
    }

    fn get_file_name(&self) -> String {
        format!("{}_{}", self.id.unwrap(), self.name)
    }

    fn get_insert_commit_message(&self) -> String {
        format!(
            "adding: [User: (id: {}, name: {}, email: {}, role: {})]",
            self.id.unwrap(),
            self.name,
            self.email,
            self.role.to_string()
        )
    }

    fn get_update_commit_message(&self) -> String {
        format!(
            "updating: [User: (id: {}, name: {}, email: {}, role: {})]",
            self.id.unwrap(),
            self.name,
            self.email,
            self.role.to_string()
        )
    }
}

impl User {
    pub fn new(name: &str, email: &str, password: &str, role: UserRole) -> Self {
        let create_date: chrono::DateTime<chrono::Utc> = chrono::Utc::now();

        Self {
            id: None,
            name: name.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            create_date,
            role,
        }
    }

    pub fn username(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn check_password(&self, password: &str) -> bool {
        self.password == *password
    }

    pub fn set_password(&mut self, password: &str) {
        self.password = password.to_string();
    }
}
