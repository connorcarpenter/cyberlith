use auth_server_types::UserId;

use crate::session_servers::SessionServerId;

pub struct UsersState {}

impl UsersState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn disconnected(&mut self, session_server_id: SessionServerId, user_id: UserId) {
        // TODO
    }
}
