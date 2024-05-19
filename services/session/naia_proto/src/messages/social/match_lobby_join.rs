use naia_bevy_shared::Message;

use social_server_types::MatchLobbyId;

#[derive(Message)]
pub struct MatchLobbyJoin {
    pub match_id: MatchLobbyId,
}

impl MatchLobbyJoin {
    pub fn new(match_id: MatchLobbyId) -> Self {
        Self {
            match_id,
        }
    }
}
