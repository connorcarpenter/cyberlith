use naia_serde::SerdeInternal as Serde;

#[derive(Serde, PartialEq, Clone)]
pub struct LoginResponse {
    pub token: String,
}

impl LoginResponse {
    pub fn new(token: &str) -> Self {
        Self { token: token.to_string() }
    }
}