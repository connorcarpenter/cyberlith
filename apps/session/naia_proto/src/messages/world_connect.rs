
use naia_bevy_shared::Message;

#[derive(Message)]
pub struct WorldConnectToken {
    pub world_server_public_webrtc_url: String,
    pub login_token: String,
}

impl WorldConnectToken {
    pub fn new(public_webrtc_url: &str, token: &str) -> Self {
        Self {
            world_server_public_webrtc_url: public_webrtc_url.to_string(),
            login_token: token.to_string(),
        }
    }
}
