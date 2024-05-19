use naia_bevy_shared::Message;

#[derive(Message)]
pub struct MatchLobbySendMessage {
    pub message: String,
}

impl MatchLobbySendMessage {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
