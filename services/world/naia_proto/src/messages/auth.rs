use naia_bevy_shared::Message;

#[derive(Message)]
pub struct Auth {
    pub login_token: String,
}

impl Auth {
    pub fn new(login_token: &str) -> Self {
        Self {
            login_token: login_token.to_string(),
        }
    }
}
