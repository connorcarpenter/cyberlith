use auth_server_types::UserId;
use social_server_types::MatchLobbyId;

use crate::session_servers::SessionServerId;

pub struct MatchLobbiesState {}

impl MatchLobbiesState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create(
        &mut self,
        session_instance_id: SessionServerId,
        match_name: &str,
        creator_user_id: UserId
    ) -> MatchLobbyId {
        // TODO
        MatchLobbyId::new(0)
    }

    pub fn join(&mut self, session_server_id: SessionServerId, match_lobby_id: MatchLobbyId, joining_user_id: UserId) {
        // TODO
    }

    pub fn leave(&mut self, session_server_id: SessionServerId, leaving_user_id: UserId) {
        // TODO
    }

    pub fn send_message(&mut self, session_server_id: SessionServerId, user_id: UserId, message: &str) {
        // TODO
    }
}