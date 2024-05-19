use auth_server_types::UserId;
use social_server_types::MatchLobbyId;

pub struct MatchLobbiesState {

}

impl MatchLobbiesState {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn create(&mut self, match_name: &str, creator_user_id: UserId) -> MatchLobbyId {
        // TODO
        MatchLobbyId::new(0)
    }

    pub fn join(&mut self, match_lobby_id: MatchLobbyId, joining_user_id: UserId) {
        // TODO
    }

    pub fn leave(&mut self, leaving_user_id: UserId) {
        // TODO
    }

    pub fn send_message(&mut self, user_id: UserId, message: &str) {
        // TODO
    }
}