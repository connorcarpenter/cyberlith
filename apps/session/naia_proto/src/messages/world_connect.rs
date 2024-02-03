
use naia_bevy_shared::Message;

#[derive(Message)]
pub struct WorldConnectToken {
    pub world_server_addr: String,
    pub world_server_port: u16,
    pub token: String,
}

impl WorldConnectToken {
    pub fn new(addr: &str, port: u16, token: &str) -> Self {
        Self {
            world_server_addr: addr.to_string(),
            world_server_port: port,
            token: token.to_string(),
        }
    }
}
