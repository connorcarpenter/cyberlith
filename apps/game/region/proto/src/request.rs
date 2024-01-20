use naia_serde::SerdeInternal as Serde;

#[derive(Serde, PartialEq, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    pub fn new(username: &str, password: &str) -> Self {
        Self { username: username.to_string(), password: password.to_string() }
    }
}