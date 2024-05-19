use naia_bevy_shared::Message;

#[derive(Message)]
pub struct GlobalChatSendMessage {
    pub message: String,
}

impl GlobalChatSendMessage {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
