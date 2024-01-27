use std::net::SocketAddr;

use naia_bevy_shared::Message;
use serde::SerdeSocketAddr;

#[derive(Message)]
pub struct WorldConnectToken {
    pub world_server_addr: SerdeSocketAddr,
    pub token: String,
}

impl WorldConnectToken {
    pub fn new(addr: SocketAddr, token: &str) -> Self {
        Self {
            world_server_addr: SerdeSocketAddr::new(addr),
            token: token.to_string(),
        }
    }
}
