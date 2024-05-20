use naia_bevy_shared::Message;

use auth_server_types::UserId;

#[derive(Message)]
pub struct GlobalChatRecvMessage {
    pub user_id: UserId,
    pub message: String,
}

impl GlobalChatRecvMessage {
    pub fn new(user_id: UserId, message: &str) -> Self {
        Self {
            user_id,
            message: message.to_string(),
        }
    }
}
