use auth_server_types::UserId;

pub struct GlobalChatState {}

impl GlobalChatState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn send_message(&mut self, user_id: UserId, message: &str) {
        // TODO
    }
}
