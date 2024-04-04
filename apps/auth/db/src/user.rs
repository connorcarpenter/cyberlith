
use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
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

// user
#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    id: UserId,
    name: String,
    email: String,
    password: String,
    date_joined: chrono::DateTime<chrono::Utc>,
    role: UserRole,
}

impl DbRowValue for User {
    type Key = UserId;

    fn get_key(&self) -> <Self as DbRowValue>::Key {
        self.id
    }
}

impl User {
    pub fn new(
        id: u64,
        name: &str,
        email: &str,
        password: &str,
        date_joined: chrono::DateTime<chrono::Utc>,
        role: UserRole,
    ) -> Self {
        Self {
            id: UserId::new(id),
            name: name.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            date_joined,
            role,
        }
    }
}