use naia_bevy_shared::Message;

#[derive(Message)]
pub struct WorldConnectToken {
    pub login_token: String,
}

impl WorldConnectToken {
    pub fn new(token: &str) -> Self {
        Self {
            login_token: token.to_string(),
        }
    }
}
