pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}