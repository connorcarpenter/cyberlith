
use naia_bevy_shared::Message;

#[derive(Message)]
pub struct WorldConnectToken {
    pub world_server_public_url: String,
    pub token: String,
}

impl WorldConnectToken {
    pub fn new(public_url: &str, token: &str) -> Self {
        Self {
            world_server_public_url: public_url.to_string(),
            token: token.to_string(),
        }
    }
}
