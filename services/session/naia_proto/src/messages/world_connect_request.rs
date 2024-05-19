use naia_bevy_shared::Message;

#[derive(Message)]
pub struct WorldConnectRequest;

impl WorldConnectRequest {
    pub fn new() -> Self {
        Self
    }
}
