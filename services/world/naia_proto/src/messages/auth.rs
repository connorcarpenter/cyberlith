use naia_bevy_shared::Message;

#[derive(Message)]
pub struct Auth {
    pub access_token: Option<String>,
    pub login_token: String,
}

impl Auth {
    pub fn new(access_token: Option<&str>, login_token: &str) -> Self {
        Self {
            access_token: access_token.map(|s| s.to_string()),
            login_token: login_token.to_string(),
        }
    }
}
